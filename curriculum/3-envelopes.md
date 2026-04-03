# Chunk 3 — Envelopes

**Day budget:** 5 days (Days 21–25)
**Status:** ⬜ Not started

### What does it do?
An envelope shapes a value over time in response to a trigger (gate on/off). The most common shape is ADSR: a fast rise (attack), a slope down to a held level (decay → sustain), and a fade to zero when released. Envelopes control amplitude (volume shape of a note), filter cutoff (brightness over time), pitch, or any other parameter. They are what make a synthesized note feel like a real musical event instead of a static tone.

### What are the knobs?
- **Attack** (seconds) — time to ramp from 0.0 to 1.0 when the gate opens. 0.001–0.01s for percussive, 0.5–2.0s for pads.
- **Decay** (seconds) — time to fall from 1.0 down to the sustain level. 0.05–0.5s typical.
- **Sustain** (level, 0.0–1.0) — the held level while the gate stays open. This is a **level**, not a time. 0.0 = percussive (no sustain), 1.0 = organ-like (no decay).
- **Release** (seconds) — time to fade from sustain level to 0.0 after gate closes. 0.05–2.0s typical.
- **Gate** (bool) — on = note held, off = note released.

### How does it connect?
- **Input:** gate signal (on/off from keyboard, sequencer, or timer)
- **Output:** control signal, 0.0 to 1.0
- **Typical chains:**
  - Amplitude VCA: `osc.process() * amp_env.process()` — shapes volume
  - Filter modulation: `base_cutoff + filter_env.process() * depth` — shapes brightness
  - Pitch modulation: `base_freq * (1.0 + pitch_env.process() * semitones)` — pitch sweep on attack
- **Key point:** The envelope implements `Process` (it generates a signal), not `Transform`. It produces a control value that you multiply or add elsewhere.

### Reference material
- DaisySP: `Adsr` class
- MI Peaks: envelope generator with exponential curves
- [Sound on Sound: Synth Secrets — Envelopes](https://www.soundonsound.com/techniques/whats-envelope)

### Daily steps

#### Day 3.1 — Linear ramp (the simplest envelope)
**Goal:** Understand that an envelope is just a value moving from A to B, one sample at a time.

Before ADSR, strip it to the minimum: a value that ramps from 0.0 to 1.0 over N seconds. This is a one-stage "attack only" envelope. It introduces the core mechanic — increment a level by a rate each sample — without any stage-switching complexity.

```rust
pub struct Ramp {
    level: f32,
    rate: f32,         // per-sample increment
    target: f32,
    sample_rate: f32,
}

impl Ramp {
    pub fn new(sample_rate: f32) -> Self {
        Self { level: 0.0, rate: 0.0, target: 1.0, sample_rate }
    }

    /// Set ramp time in seconds.
    pub fn set_time(&mut self, seconds: f32) {
        if seconds <= 0.0 {
            self.rate = 0.0;
            self.level = self.target;
        } else {
            self.rate = 1.0 / (seconds * self.sample_rate);
        }
    }

    pub fn trigger(&mut self) {
        self.level = 0.0;
    }
}

impl Process for Ramp {
    fn process(&mut self) -> f32 {
        if self.level < self.target {
            self.level += self.rate;
            if self.level > self.target { self.level = self.target; }
        }
        self.level
    }
}
```

This is example-only code — it's a stepping stone, not a library struct. The AR envelope on Day 3.2 supersedes it.

**Key insight:** `rate = 1.0 / (time_in_seconds * sample_rate)`. At 44100 Hz and 0.5 seconds, the rate is `1.0 / 22050 ≈ 0.0000453` per sample. Tiny, but 22050 of them add up to 1.0. This is the same seconds-to-samples conversion you've seen before.

**Exercise:** Trigger a 0.5-second ramp, multiply it by a saw oscillator's output. You hear the oscillator fade in over half a second from silence to full volume. That's an envelope shaping amplitude.

---

#### Day 3.2 — AR envelope (attack-release with a gate)
**Goal:** Introduce the gate concept and stage switching.

The gate is a boolean: on = "note is held", off = "note released." While gate is on, the envelope ramps up to 1.0 (attack). When gate goes off, it ramps back down to 0.0 (release) **from wherever it currently is**. That detail matters — if you release during the attack before reaching 1.0, the release starts from that intermediate level.

```rust
#[derive(Clone, Copy, PartialEq)]
pub enum ArStage { Idle, Attack, Release }

pub struct Ar {
    stage: ArStage,
    level: f32,
    attack_rate: f32,
    release_rate: f32,
    sample_rate: f32,
    gate: bool,
}

impl Ar {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            stage: ArStage::Idle,
            level: 0.0,
            attack_rate: 0.0,
            release_rate: 0.0,
            sample_rate,
            gate: false,
        }
    }

    pub fn set_attack(&mut self, seconds: f32) {
        self.attack_rate = if seconds > 0.0 {
            1.0 / (seconds * self.sample_rate)
        } else { 1.0 };
    }

    pub fn set_release(&mut self, seconds: f32) {
        self.release_rate = if seconds > 0.0 {
            1.0 / (seconds * self.sample_rate)
        } else { 1.0 };
    }

    pub fn set_gate(&mut self, gate: bool) {
        self.gate = gate;
    }
}

impl Process for Ar {
    fn process(&mut self) -> f32 {
        match self.stage {
            ArStage::Idle => {
                if self.gate { self.stage = ArStage::Attack; }
            }
            ArStage::Attack => {
                if !self.gate {
                    self.stage = ArStage::Release;
                } else {
                    self.level += self.attack_rate;
                    if self.level >= 1.0 {
                        self.level = 1.0;
                    }
                }
            }
            ArStage::Release => {
                self.level -= self.release_rate;
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.stage = ArStage::Idle;
                }
                if self.gate { self.stage = ArStage::Attack; }
            }
        }
        self.level
    }
}
```

**Key insight:** The stage machine is the core pattern. Every envelope is: check what stage you're in, advance the level, check if the transition condition is met, move to the next stage. ADSR just adds two more stages to this exact pattern.

**Exercise:** Set attack = 0.3s, release = 0.5s. Use a timed gate (on for 1 second, off for 1 second, repeating) multiplied by a saw oscillator. You hear: fade-in over 0.3s, hold at full volume, fade-out over 0.5s, silence, repeat. Then try a very short gate (50ms) with the same 0.3s attack — the note releases before attack finishes. Listen to how the peak is lower. This is correct and musically important.

---

#### Day 3.3 — Full ADSR (linear)
**Goal:** Extend AR to the full four-stage ADSR — the classic synthesizer envelope.

ADSR adds **decay** (ramp from 1.0 down to sustain level after attack completes) and **sustain** (a held level while the gate stays open). The four stages:

1. **Attack:** level ramps from 0.0 → 1.0
2. **Decay:** level ramps from 1.0 → sustain_level
3. **Sustain:** level holds at sustain_level while gate is on
4. **Release:** level ramps from current level → 0.0

This is the main library struct. Create `src/envelope/adsr.rs`:

```rust
#[derive(Clone, Copy, PartialEq)]
pub enum Stage { Idle, Attack, Decay, Sustain, Release }

pub struct Adsr {
    stage: Stage,
    level: f32,
    attack_rate: f32,
    decay_rate: f32,
    sustain_level: f32,
    release_rate: f32,
    sample_rate: f32,
    gate: bool,
}
```

Setters follow the same pattern: `set_attack(seconds)`, `set_decay(seconds)`, `set_sustain(level)`, `set_release(seconds)`, `set_gate(bool)`.

The process method extends the AR pattern:

```rust
impl Process for Adsr {
    fn process(&mut self) -> f32 {
        match self.stage {
            Stage::Idle => {
                if self.gate { self.stage = Stage::Attack; }
            }
            Stage::Attack => {
                self.level += self.attack_rate;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = Stage::Decay;
                }
                if !self.gate { self.stage = Stage::Release; }
            }
            Stage::Decay => {
                self.level -= self.decay_rate;
                if self.level <= self.sustain_level {
                    self.level = self.sustain_level;
                    self.stage = Stage::Sustain;
                }
                if !self.gate { self.stage = Stage::Release; }
            }
            Stage::Sustain => {
                self.level = self.sustain_level;
                if !self.gate { self.stage = Stage::Release; }
            }
            Stage::Release => {
                self.level -= self.release_rate;
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.stage = Stage::Idle;
                }
                if self.gate { self.stage = Stage::Attack; }
            }
        }
        self.level
    }
}
```

**Key insight:** Sustain is a **level** (0.0–1.0), not a time. Attack, decay, and release are times. Sustain = 0.0 with short decay = percussive. Sustain = 1.0 eliminates the decay stage entirely. These two extremes cover most musical use cases.

**Exercise:** Two patches to compare:
1. **Pluck:** attack = 5ms, decay = 200ms, sustain = 0.0, release = 100ms. Gate on for 300ms. Multiply by saw osc. Sharp attack, quick decay to silence — a plucked string.
2. **Pad:** attack = 500ms, decay = 300ms, sustain = 0.7, release = 1s. Gate on for 2 seconds. The sound swells, settles to 70%, then fades.

---

#### Day 3.4 — Exponential curves (making it sound right)
**Goal:** Replace linear ramps with exponential curves — the difference between "technically correct" and "musically correct."

Linear envelopes sound unnatural because human hearing is logarithmic. A linear decay sounds like it hangs on too long before dropping off. Real analog envelopes use RC circuits that produce exponential curves — fast at the start, slow at the end for decay/release.

The trick: use the same one-pole formula from Chunk 2. Instead of adding a fixed rate, chase a target exponentially:

```rust
// Linear (Day 3.3):
self.level += self.attack_rate;

// Exponential:
self.level += self.attack_coeff * (attack_target - self.level);
```

The problem: an exponential approach never reaches its target. Solution: overshoot. For attack, aim above 1.0 (e.g. 1.3). For decay/release, aim below 0.0 (e.g. -0.3). The level crosses the real threshold in finite time.

The coefficient from time:
```rust
fn time_to_coeff(seconds: f32, sample_rate: f32) -> f32 {
    if seconds <= 0.0 { return 1.0; }
    1.0 - (-1.0 / (seconds * sample_rate)).exp()
}
```

In the attack stage:
```rust
const OVERSHOOT: f32 = 0.3;

Stage::Attack => {
    let target = 1.0 + OVERSHOOT;
    self.level += self.attack_coeff * (target - self.level);
    if self.level >= 1.0 {
        self.level = 1.0;
        self.stage = Stage::Decay;
    }
    if !self.gate { self.stage = Stage::Release; }
}
```

Apply the same pattern to decay (target = `sustain_level - OVERSHOOT`) and release (target = `0.0 - OVERSHOOT`).

**Key insight:** `state += coeff * (target - state)` is the most reusable formula in all of DSP. You have now used it for: (1) lowpass filtering, (2) parameter smoothing, (3) envelope curves. They are all the same thing — an exponential approach to a target.

**Exercise:** Compare the linear ADSR from Day 3.3 with the exponential version using identical time settings. The exponential version sounds smoother and more "analog." The decay fades naturally instead of linearly. Render both to WAV and compare the shapes in Audacity — straight lines vs curves.

---

#### Day 3.5 — Envelope as modulation source (connecting everything)
**Goal:** Wire the ADSR to both amplitude and filter cutoff — the complete subtractive synth voice.

In a real synthesizer, you typically have two envelopes: one for amplitude (VCA envelope) and one for filter cutoff (filter envelope). They can have completely different ADSR settings. The amplitude envelope shapes volume over time. The filter envelope shapes brightness over time. Together they create evolving timbre.

```rust
let mut osc = BlepOscillator::new(sample_rate, Waveform::Saw);
let mut filter = Svf::new(sample_rate);
let mut amp_env = Adsr::new(sample_rate);
let mut filter_env = Adsr::new(sample_rate);
let mut cutoff_smoother = OnePole::new(sample_rate);
cutoff_smoother.set_cutoff(200.0);

// Plucky bass
amp_env.set_attack(0.005);   amp_env.set_decay(0.3);
amp_env.set_sustain(0.0);    amp_env.set_release(0.1);

filter_env.set_attack(0.001); filter_env.set_decay(0.2);
filter_env.set_sustain(0.0);  filter_env.set_release(0.1);

// In audio callback:
let base_cutoff = 200.0;
let env_depth = 4000.0;
let raw_cutoff = base_cutoff + filter_env.process() * env_depth;
let smooth_cutoff = cutoff_smoother.process(raw_cutoff);
filter.set_params(smooth_cutoff, 2.0);

let audio = filter.process(osc.process());
let output = audio * amp_env.process();
```

**Key insight:** The envelope is a control signal, not an audio processor. Its output (0.0–1.0) gets multiplied or added elsewhere. For amplitude: `audio * env`. For filter: `base + env * depth`. For pitch: `base_freq * (1.0 + env * amount)`. The same `Adsr` struct works for all three — only the wiring changes.

**Exercise:** Three patches to audition:
1. **Plucky bass:** Both envelopes with sustain = 0, short decay. Filter sweep on attack gives it bite.
2. **Pad:** Long attack (1s), sustain = 0.8, long release (2s). Filter envelope with moderate depth for a slow brightness swell.
3. **Percussive hit:** Zero attack, very short decay (50ms), sustain = 0 on both. The sound is entirely shaped by the decay.

Then try: fast filter decay with slow amplitude release — "bright attack, warm sustain." This is the hallmark of classic analog pads.

---

### Module checklist
- [ ] `Adsr` with exponential curves and `set_attack/decay/sustain/release/gate` setters
- [ ] Implements `Process` (returns 0.0..1.0), `Reset`, `SetSampleRate`
- [ ] Stage enum: Idle, Attack, Decay, Sustain, Release
- [ ] Retrigger works: gate on during release restarts attack from current level
- [ ] Exponential curves with overshoot for natural-sounding segments
- [ ] Example: oscillator → SVF → VCA, with separate ADSRs for filter and amplitude
- [ ] You can hear the difference between linear and exponential envelope curves
- [ ] You understand: sustain is a level not a time, release starts from current level, `state += coeff * (target - state)` is the same formula as OnePole filtering
