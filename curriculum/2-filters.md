# Chunk 2 — Filters

**Day budget:** 8 days (Days 13–20)
**Status:** ⬜ Not started

### What does it do?
A filter shapes the frequency content of a signal. It boosts or cuts certain frequency bands. The most common types: **lowpass** (lets lows through, cuts highs — the classic synth sweep sound), **highpass** (the opposite), **bandpass** (lets a band through), **notch** (cuts a band).

### What are the knobs?
- **Cutoff frequency** (Hz) — where the filter starts to act
- **Resonance / Q** — how much the filter peaks at the cutoff frequency. Low Q is gentle, high Q is sharp and can self-oscillate
- **Filter type** — lowpass, highpass, bandpass, notch
- **Drive / saturation** — some filters add harmonic coloring (especially ladder filters)

### How does it connect?
- **Input:** audio signal (from an oscillator, or anything)
- **Output:** filtered audio signal
- **Typical chain:** Oscillator → Filter → Envelope VCA → Output
- Modulate the cutoff with an envelope or LFO for the classic sweeping synth sound

### Reference material
- DaisySP: `Svf` class (state variable filter — the workhorse)
- MI Elements/Peaks: use SVF extensively
- Cytomic blog: excellent SVF write-up by Andy Simper
- [EarLevel: Biquad filters](https://www.earlevel.com/main/2012/11/26/biquad-c-source-code/)

### Daily steps

#### Day 2.1 — The one-pole lowpass (the simplest filter)
**Goal:** Understand filtering as weighted memory, in two lines of code.

```rust
pub struct OnePole {
    state: f32,
    coefficient: f32,  // 0..1, higher = slower response = lower cutoff
}

impl OnePole {
    /// cutoff in Hz, sample_rate in Hz
    pub fn set_cutoff(&mut self, cutoff: f32, sample_rate: f32) {
        // This formula gives an approximate coefficient for the desired cutoff
        let rc = 1.0 / (std::f32::consts::TAU * cutoff);
        let dt = 1.0 / sample_rate;
        self.coefficient = dt / (rc + dt);
    }
}

impl Process for OnePole {
    fn process_sample(&mut self, input: f32) -> f32 {
        // state = state + coeff * (input - state)
        // = lerp(state, input, coeff)
        self.state += self.coefficient * (input - self.state);
        self.state
    }
}
```

This is an **exponential moving average**. It's also called a **leaky integrator**. The state "chases" the input but lags behind — that lag is the filtering effect.

**Key insight:** A one-pole filter is just interpolation between the last output and the current input. Higher coefficient = closer to input = higher cutoff = less filtering. This same structure is used for parameter smoothing everywhere.

**Parameter smoothing — use this immediately.** Now that you have a one-pole filter, you have the tool to fix zipper noise. Wrap any parameter that changes over time in its own `OnePole` instance, and feed the raw target value through it each sample before passing it to the module:

```rust
// Instead of:
svf.set_cutoff(lfo.process() * 2000.0 + 500.0, sample_rate); // clicks on fast LFO

// Do this:
let raw_cutoff = lfo.process() * 2000.0 + 500.0;
let smooth_cutoff = cutoff_smoother.process(raw_cutoff); // OnePole at ~5–20Hz
svf.set_cutoff(smooth_cutoff, sample_rate);              // silky smooth
```

A smoothing time constant of 5–20ms (equivalent to a cutoff of ~50–200Hz on the smoothing filter) is typical. For slow parameters like filter sweeps, 50ms feels natural. For fast envelopes you may want 1–5ms to preserve the attack transient.

**Exercise:** Use a one-pole filter to smooth your LFO output — notice how it rounds off the corners of a square LFO wave into a gentle curve. Then apply the same smoother to the cutoff frequency parameter of the SVF in Day 2.5 and compare the sound with and without it.

---

#### Day 2.2 — One-pole highpass and DC blocking
**Goal:** Understand that highpass = input minus lowpass.

```rust
// Highpass = signal - lowpass(signal)
fn process_highpass(&mut self, input: f32) -> f32 {
    let lp = self.process_sample(input); // your lowpass state
    input - lp
}
```

This is not obvious but it's profound: subtracting the "smooth part" leaves only the "sharp part". This is why highpass filters have a complementary relationship with lowpass filters.

A **DC blocker** is a highpass with a very low cutoff (~10Hz) — it removes any constant offset in the signal that can cause clicks or distortion.

**Exercise:** Add DC offset to a sine wave (`out + 0.5`) and listen — it changes the waveform asymmetrically. Run it through a DC blocker and the offset disappears.

---

#### Day 2.3 — Biquad filter (the general-purpose workhorse)
**Goal:** Understand the biquad as a configurable 2-pole filter.

The biquad is the standard filter you'll reach for most often. It has 5 coefficients (a0, a1, a2, b1, b2) that determine its behavior, and you recalculate them when cutoff or Q changes.

```rust
pub struct Biquad {
    // Coefficients
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    // State (the "memory" of the filter)
    x1: f32, x2: f32,
    y1: f32, y2: f32,
}

impl Biquad {
    pub fn process_sample(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
                            - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1; self.x1 = x;
        self.y2 = self.y1; self.y1 = y;
        y
    }
}
```

The coefficient calculation formulas are on EarLevel (link above) — copy them. You don't need to derive them, just know which formula gives you LP vs HP vs BP.

**Exercise:** Implement lowpass and highpass modes. Sweep the cutoff from 200Hz to 8000Hz and listen to the classic synth sweep.

---

#### Day 2.4 — Resonance (Q)
**Goal:** Understand what Q sounds like and why it matters.

Resonance makes the filter peak at the cutoff frequency. At Q=0.707 (1/√2), the filter is maximally flat (Butterworth — the default "neutral" Q). Above that, you get a resonant peak. Near Q=1.0 for a biquad, the filter approaches self-oscillation (it starts ringing on its own).

**Exercise:** Set a fixed cutoff of 800Hz. Slowly increase Q from 0.5 to 5.0. Notice the peak forming, then the resonant ringing. This is what makes filter sweeps exciting.

**Key insight:** The resonance peak is what gives synth filters their character. A Moog ladder filter has a specific resonance character that's very different from an SVF's resonance, even at the same Q value.

---

#### Day 2.5 — State Variable Filter (SVF)
**Goal:** Understand why the SVF is the preferred filter for synth work.

The SVF gives you lowpass, highpass, and bandpass **simultaneously from one computation**, and it's more stable at high resonance than the biquad. This is why DaisySP and MI both use it heavily.

```rust
pub struct Svf {
    lp: f32,   // lowpass state
    bp: f32,   // bandpass state
    // hp is computed: hp = input - lp - q * bp
    f: f32,    // frequency coefficient
    q: f32,    // resonance (1/Q, so higher = less resonant)
}

impl Svf {
    pub fn set_params(&mut self, cutoff: f32, resonance: f32, sample_rate: f32) {
        self.f = 2.0 * (std::f32::consts::PI * cutoff / sample_rate).sin();
        self.q = 1.0 / resonance; // note: inverted
    }

    /// Returns (lowpass, bandpass, highpass)
    pub fn process_sample(&mut self, input: f32) -> (f32, f32, f32) {
        let hp = input - self.lp - self.q * self.bp;
        self.bp += self.f * hp;
        self.lp += self.f * self.bp;

        // Flush denormals: subnormal floats in feedback loops cause CPU spikes.
        // When signal decays to silence, tiny values (1e-40) hit software emulation.
        // This clamps them to zero — one line, mandatory in every feedback structure.
        if self.lp.abs() < 1e-15 { self.lp = 0.0; }
        if self.bp.abs() < 1e-15 { self.bp = 0.0; }

        (self.lp, self.bp, hp)
    }
}
```

> **Why denormals matter here:** The SVF has two integrator state variables (`lp` and `bp`) that feed back into each other every sample. When the input signal goes silent, these states decay exponentially toward zero — but they never quite reach it in floating point. Instead they enter the subnormal range, where the CPU switches from hardware float arithmetic to a software emulation routine that is roughly 100x slower. Your CPU usage jumps from ~1% to ~100% and you have no idea why. The two `if` lines above clamp to zero at the noise floor, which is well below the audible threshold (~96dB below full scale). Add this to every IIR filter and feedback delay you build.

**Exercise:** Connect a saw oscillator → SVF. Listen to LP, BP, HP outputs. Then sweep frequency and resonance together. This is the sound of analog synthesis.

---

#### Day 2.6 — Filter modulation
**Goal:** Connect your Chunk 1 modules to make a real synth patch.

Now everything starts connecting. Build a small signal chain entirely in code:

```rust
let osc_out = osc.process();         // Chunk 1
let env_out = envelope.process();    // Will formalize in Chunk 3
let lfo_out = lfo.process();         // Chunk 1

// Modulate filter cutoff with envelope + LFO
let cutoff = 200.0 + env_out * 3000.0 + lfo_out * 500.0;
svf.set_cutoff(cutoff, sample_rate);

let (lp, _, _) = svf.process_sample(osc_out);
output = lp;
```

**Key insight:** This is what a modular synthesizer is — a set of modules with their parameter inputs wired to control signals. Your Rust structs *are* the modules.

---

#### Day 2.7 — Ladder filter (optional / stretch)
**Goal:** Understand the Moog ladder filter character.

The transistor ladder filter (Moog) is a cascade of 4 one-pole lowpass filters with feedback. It has a famously "warm" character. Implementing a basic version deepens your understanding of filter topology. Reference the DaisySP `MoogLadder` class.

This is optional — skip if you'd rather move on and come back later.

---

#### Day 2.8 — Review and clean up
- Implement `Process`-compatible interface on SVF (decide: return LP by default, expose mode enum)
- Write `examples/chunk2_filters.rs` — interactive patch with osc + SVF + LFO modulating cutoff
- Update CURRICULUM.md status to ✅

### Module checklist
- [ ] `OnePole` (lowpass and highpass) with `Transform` trait
- [ ] DC blocker utility function
- [ ] `Biquad` with LP/HP/BP modes and Q
- [ ] `Svf` with all three outputs and denormal flush on state variables
- [ ] All implement `Reset` + `SetSampleRate`
- [ ] Parameter smoothing: `OnePole` used to smooth cutoff before passing to SVF — no zipper noise on LFO sweep
- [ ] Example runs: osc → SVF → output with LFO modulating cutoff smoothly
- [ ] You can hear the difference between an unsmoothed and smoothed parameter sweep
- [ ] You understand why the denormal flush lines are in every feedback structure