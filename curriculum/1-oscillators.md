# Chunk 1 — Oscillators

**Day budget:** 8 days (Days 5–12)
**Status:** ⬜ Not started

### What does it do?
An oscillator generates a repeating periodic signal. It is the primary sound source in synthesis. Different waveshapes have different harmonic content: sine is pure, saw is bright and buzzy, square is hollow, triangle sits between them.

### What are the knobs?
- **Frequency** (Hz) — how fast the wave repeats
- **Amplitude** — how loud
- **Phase offset** — where in the cycle it starts (useful for modulation)
- **Waveshape** — which waveform (for multi-shape oscillators)

### How does it connect?
- **Input:** nothing (it generates signal) or a frequency modulation signal
- **Output:** audio signal (-1.0 to 1.0) into a filter, envelope, or directly to output
- **Typical chain:** Oscillator → Filter → Envelope → Output

### Reference material
- DaisySP: `Oscillator` class (`daisysp/Source/Synthesis/oscillator.h`)
- MI Braids: `braids/analog_oscillator.cc` — excellent polyBLEP reference
- Blog: [The polyBLEP method by Martin Finke](http://www.martin-finke.de/blog/articles/audio-plugins-018-polyblep-oscillator/)

### Daily steps

#### Day 1.1 — Sine wave from scratch
**Goal:** Understand the phase accumulator pattern — the heartbeat of every oscillator.

A sine oscillator has one piece of state: `phase`, a value from 0.0 to 1.0 that advances each sample.

```rust
// src/oscillator/sine.rs
pub struct Sine {
    phase: f32,
    phase_increment: f32,
}

impl Sine {
    pub fn new(sample_rate: f32) -> Self {
        Self { phase: 0.0, phase_increment: 0.0 }
    }

    pub fn set_frequency(&mut self, freq: f32, sample_rate: f32) {
        self.phase_increment = freq / sample_rate;
    }
}

impl Process for Sine {
    fn process(&mut self) -> f32 {
        let out = (self.phase * std::f32::consts::TAU).sin();
        self.phase = (self.phase + self.phase_increment) % 1.0;
        out
    }
}
```

Key insight: `phase_increment = freq / sample_rate` — this is how you convert Hz into "fraction of a cycle per sample". All oscillators use this.

**Exercise:** Change frequency by calling `set_frequency` mid-playback. Notice it changes instantly with no click — because phase is preserved.

---

#### Day 1.2 — Naive saw, square, triangle
**Goal:** Understand that waveshape = different harmonic content, and why "naive" is a problem.

A naive saw wave is just the phase ramp itself (0.0 to 1.0, then wrap):
```rust
// Naive saw — sounds buzzy, has aliasing at high frequencies
let out = self.phase * 2.0 - 1.0; // rescale to -1..1
```

A naive square:
```rust
let out = if self.phase < 0.5 { 1.0 } else { -1.0 };
```

A naive triangle:
```rust
let out = if self.phase < 0.5 {
    self.phase * 4.0 - 1.0
} else {
    3.0 - self.phase * 4.0
};
```

**Exercise:** Implement a `NaiveOscillator` with a `Waveform` enum. Play it at 200Hz (sounds fine) then at 4000Hz (notice the aliasing — a harsh, metallic quality). That problem is what Day 1.5 solves.

**What is aliasing?** When a waveform has sharp edges (saw, square), it contains frequencies above Nyquist (sample_rate / 2). Those fold back down into the audible range as false tones. Sine has no sharp edges, so no aliasing.

---

#### Day 1.3 — Wavetable oscillator
**Goal:** Understand lookup tables as a speed/quality tradeoff.

Instead of computing `sin()` every sample (slow on embedded), precompute a table of 2048 sine values and look them up.

```rust
const TABLE_SIZE: usize = 2048;

pub struct Wavetable {
    table: [f32; TABLE_SIZE],
    phase: f32,
    phase_increment: f32,
}

impl Wavetable {
    pub fn new_sine(sample_rate: f32) -> Self {
        let mut table = [0.0f32; TABLE_SIZE];
        for i in 0..TABLE_SIZE {
            table[i] = (i as f32 / TABLE_SIZE as f32 * std::f32::consts::TAU).sin();
        }
        Self { table, phase: 0.0, phase_increment: 0.0 }
    }
}

impl Process for Wavetable {
    fn process(&mut self) -> f32 {
        // Linear interpolation between table entries
        let idx = self.phase * TABLE_SIZE as f32;
        let idx0 = idx as usize % TABLE_SIZE;
        let idx1 = (idx0 + 1) % TABLE_SIZE;
        let frac = idx - idx.floor();
        let out = self.table[idx0] * (1.0 - frac) + self.table[idx1] * frac;
        self.phase = (self.phase + self.phase_increment) % 1.0;
        out
    }
}
```

**Key insight:** Linear interpolation between table entries removes the stepping artifacts you'd get from just rounding the index.

**Exercise:** Fill the table with a hand-drawn shape (e.g. a few sine harmonics added together) and notice how the timbre changes.

---

#### Day 1.4 — Frequency modulation (FM) basics
**Goal:** Understand how oscillators modulate each other.

Connect the output of one oscillator to the frequency input of another:

```rust
let modulator_out = modulator.process(); // -1.0 to 1.0
let modulated_freq = carrier_base_freq + modulator_out * mod_depth;
carrier.set_frequency(modulated_freq, sample_rate);
let out = carrier.process();
```

This is the basis of FM synthesis. At audio-rate modulation (mod freq > ~20Hz), you get complex harmonic sidebands. At LFO rate (< 20Hz), it's just vibrato.

**Exercise:** Set modulator to 2Hz, depth to 20Hz → vibrato. Then push modulator to 200Hz, depth to 500Hz → FM bell/metallic tone.

---

#### Day 1.5 — PolyBLEP anti-aliasing
**Goal:** Fix the aliasing problem from Day 1.2 without needing complex math.

PolyBLEP adds a small correction at the discontinuity (the edge/wrap point) of the waveform to smooth it out. You don't need to deeply understand *why* it works — just know it's a residual correction applied at the phase reset point.

```rust
/// Apply polyBLEP correction around a discontinuity.
/// `phase` is current phase (0..1), `increment` is phase_increment.
/// Call this once per discontinuity per sample.
fn poly_blep(phase: f32, increment: f32) -> f32 {
    if phase < increment {
        let t = phase / increment;
        2.0 * t - t * t - 1.0
    } else if phase > 1.0 - increment {
        let t = (phase - 1.0) / increment;
        t * t + 2.0 * t + 1.0
    } else {
        0.0
    }
}
```

For a saw wave, apply it at the wrap point:
```rust
let mut out = self.phase * 2.0 - 1.0;
out -= poly_blep(self.phase, self.phase_increment);
```

For square, apply at both transitions (phase 0.0 and 0.5).

**Exercise:** Compare naive saw vs polyBLEP saw at 2000Hz. The aliasing chirping in the naive version should be largely gone. You can verify by recording both and looking at a spectrogram.

---

#### Day 1.6 — LFO (Low Frequency Oscillator)
**Goal:** Reuse everything above at sub-audio rates for modulation.

An LFO is literally the same oscillator running at 0.1–20Hz instead of 20–20000Hz. The only difference is intent: it modulates parameters instead of producing audio.

```rust
// An LFO is just a sine oscillator with a different frequency range
pub struct Lfo {
    osc: Sine,
    rate: f32,       // Hz, typically 0.01 to 20.0
    depth: f32,      // output multiplier
}

impl Lfo {
    pub fn process(&mut self) -> f32 {
        self.osc.process() * self.depth
    }
}
```

Common LFO targets: oscillator pitch (vibrato), filter cutoff (wah), amplitude (tremolo).

**Exercise:** Route an LFO at 0.5Hz into the cutoff frequency of the filter you'll build in Chunk 2. This is a classic synth patch.

---

#### Day 1.7 — Detune and unison
**Goal:** Understand how layering creates width and richness.

Run 3–7 oscillators at slightly detuned frequencies (e.g. ±10 cents spread), mix their outputs. The slight phase differences create the "supersaw" sound.

```rust
// Detune in cents: multiply frequency by 2^(cents/1200)
fn cents_to_ratio(cents: f32) -> f32 {
    (cents / 1200.0_f32).exp2()
}
```

**Exercise:** Build a `UnisonOscillator` that holds 3 `Sine` instances at -10, 0, +10 cents. Mix and notice the chorus-like beating.

---

#### Day 1.8 — Review and clean up
- Refactor all oscillators to properly implement `Process`, `Reset`, `SampleRate`
- Write `examples/chunk1_oscillators.rs` that lets you audition each type with a keypress
- Make sure `src/oscillator/mod.rs` re-exports cleanly
- Update CURRICULUM.md status to ✅

### Module checklist
- [ ] `Sine` with phase accumulator
- [ ] `NaiveOscillator` with Waveform enum
- [ ] `Wavetable` with linear interpolation
- [ ] `PolyBlepOscillator` for saw and square
- [ ] `Lfo` wrapping the above
- [ ] `UnisonOscillator` with detune
- [ ] All implement `Process` + `Reset` + `SampleRate`
- [ ] Example runs and produces audio