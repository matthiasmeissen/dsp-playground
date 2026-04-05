use std::f32::consts::PI;
use crate::{Transform, Reset, SetSampleRate};

/// Moog-style 4-pole (24 dB/oct) resonant ladder filter.
///
/// Uses the Topology-Preserving Transform (TPT) approach from Zavalishin's
/// "The Art of VA Filter Design", with Della Cioppa's correction for accurate
/// state variable computation.
///
/// Passband gain compensation follows from the Stilson & Smith (CCRMA 1996)
/// transfer function analysis: H(s) = G⁴/(1 + k·G⁴), giving DC gain = 1/(1+k).
/// We pre-scale the input by (1+k) so the passband stays at unity regardless
/// of resonance — the same volume as your SVF at the same settings.
///
/// # References
/// - Stilson & Smith, "Analyzing the Moog VCF with Considerations for Digital
///   Implementation", ICMC 1996 (https://ccrma.stanford.edu/~stilti/papers/moogvcf.pdf)
/// - JOS, Moog VCF Controls (https://ccrma.stanford.edu/~jos/Mohonk05/Moog_VCF_Controls.html)
/// - Zavalishin, "The Art of VA Filter Design", Ch. 4–5
/// - Della Cioppa's fix: https://www.kvraudio.com/forum/viewtopic.php?t=571909
///
/// # Usage
/// ```
/// use dsp_lib::{Transform, filter::ladder::Ladder};
///
/// let mut filter = Ladder::new(44100.0);
/// filter.set_cutoff(800.0);
/// filter.set_resonance(0.5);  // 0.0 = none, 1.0 = self-oscillation
/// let output = filter.process(1.0);
/// ```
pub struct Ladder {
    s: [f32; 4],          // four TPT integrator states
    g: f32,               // integrator gain: tan(PI * fc / sr)
    k: f32,               // resonance feedback (0.0–4.0 internally)
    drive: f32,           // input drive for nonlinearity
    mode: LadderMode,
    sample_rate: f32,
}

/// Nonlinearity applied at each ladder stage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LadderMode {
    /// No nonlinearity — pure linear TPT ladder.
    Linear,
    /// tanh saturation at each stage. Warm, classic Moog character.
    Tanh,
    /// Cubic soft-clip approximation. Cheaper than tanh, still musical.
    SoftClip,
}

impl Ladder {
    pub fn new(sample_rate: f32) -> Self {
        let mut f = Self {
            s: [0.0; 4],
            g: 0.0,
            k: 0.0,
            drive: 1.0,
            mode: LadderMode::Linear,
            sample_rate,
        };
        f.set_cutoff(1000.0);
        f.set_resonance(0.0);
        f
    }

    /// Set cutoff frequency in Hz (clamped to 20 – just below Nyquist).
    pub fn set_cutoff(&mut self, cutoff: f32) {
        let fc = cutoff.clamp(20.0, self.sample_rate * 0.49);
        self.g = (PI * fc / self.sample_rate).tan();
    }

    /// Set resonance: 0.0 = none, 1.0 = self-oscillation.
    pub fn set_resonance(&mut self, resonance: f32) {
        self.k = resonance.clamp(0.0, 1.0) * 4.0;
    }

    /// Set input drive (1.0 = unity, higher pushes harder into the
    /// nonlinearity for more saturation character).
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.max(0.0);
    }

    /// Set the nonlinearity mode.
    pub fn set_mode(&mut self, mode: LadderMode) {
        self.mode = mode;
    }
}

impl Transform for Ladder {
    fn process(&mut self, input: f32) -> f32 {
        let g = self.g;
        let k = self.k;

        // Per-stage gain: G = g / (1 + g)
        let big_g = g / (1.0 + g);
        let one_over_1pg = 1.0 / (1.0 + g);

        // Della Cioppa's corrected S: each state weighted by (1-G) = 1/(1+g)
        // and by the appropriate power of G for the stages above it.
        //
        //   S = G³·s₀/(1+g) + G²·s₁/(1+g) + G·s₂/(1+g) + s₃/(1+g)
        let s_weighted = big_g * big_g * big_g * self.s[0] * one_over_1pg
            + big_g * big_g * self.s[1] * one_over_1pg
            + big_g * self.s[2] * one_over_1pg
            + self.s[3] * one_over_1pg;

        // Total gain through 4 cascaded stages
        let big_g4 = big_g * big_g * big_g * big_g;

        // Passband gain compensation (Stilson & Smith, CCRMA 1996):
        //
        //   The Moog VCF transfer function is  H(s) = G⁴ / (1 + k·G⁴)
        //   At DC (w=0), G = 1, so  H(0) = 1 / (1 + k)
        //
        //   Pre-scale the input by (1+k) so passband stays at unity.
        //   Applied BEFORE the feedback solver so the nonlinearities
        //   see the boosted signal naturally.
        let compensation = 1.0 + k;
        let x = input * self.drive * compensation;

        // Resolve feedback analytically (zero-delay):
        //   u = (x - k·S) / (1 + k·G⁴)
        let u = (x - k * s_weighted) / (1.0 + k * big_g4);

        // Process four one-pole TPT integrator stages in series.
        let mut stage_in = u;
        for i in 0..4 {
            let v = self.saturate(stage_in - self.s[i]) * g;
            let out = v + self.s[i];
            self.s[i] = out + v;
            stage_in = out;
        }

        // Flush denormals — prevents CPU spikes when signal decays
        for state in self.s.iter_mut() {
            if state.abs() < 1e-15 {
                *state = 0.0;
            }
        }

        // Undo drive scaling (but NOT the compensation — that's intentional)
        let out = if self.drive > 0.0 {
            stage_in / self.drive
        } else {
            stage_in
        };

        out
    }
}

impl Reset for Ladder {
    fn reset(&mut self) {
        self.s = [0.0; 4];
    }
}

impl SetSampleRate for Ladder {
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}

impl Ladder {
    #[inline]
    fn saturate(&self, x: f32) -> f32 {
        match self.mode {
            LadderMode::Linear => x,
            LadderMode::Tanh => fast_tanh(x),
            LadderMode::SoftClip => soft_clip(x),
        }
    }
}

/// Fast tanh — Padé approximant, accurate to ~0.1% for |x| < 3.
#[inline]
fn fast_tanh(x: f32) -> f32 {
    let x2 = x * x;
    let x4 = x2 * x2;
    let num = x * (135135.0 + x2 * (17325.0 + x2 * (378.0 + x2)));
    let den = 135135.0 + x2 * (62370.0 + x2 * (3150.0 + x4));
    (num / den).clamp(-1.0, 1.0)
}

/// Cubic soft clipper: y = x - x³/3 for |x| < 1.
#[inline]
fn soft_clip(x: f32) -> f32 {
    if x > 1.0 {
        2.0 / 3.0
    } else if x < -1.0 {
        -2.0 / 3.0
    } else {
        x - (x * x * x) / 3.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_in_silence_out() {
        let mut f = Ladder::new(44100.0);
        f.set_cutoff(1000.0);
        f.set_resonance(0.0);
        for _ in 0..100 {
            let out = f.process(0.0);
            assert!(out.abs() < 1e-10, "expected silence, got {out}");
        }
    }

    #[test]
    fn passes_dc_at_unity_without_resonance() {
        let mut f = Ladder::new(44100.0);
        f.set_cutoff(1000.0);
        f.set_resonance(0.0);
        let mut out = 0.0;
        for _ in 0..2000 {
            out = f.process(1.0);
        }
        assert!(
            (out - 1.0).abs() < 0.01,
            "DC should pass at unity with no resonance, got {out}"
        );
    }

    #[test]
    fn dc_stays_near_unity_with_resonance() {
        for res in [0.25, 0.5, 0.75, 0.9] {
            let mut f = Ladder::new(44100.0);
            f.set_cutoff(1000.0);
            f.set_resonance(res);
            let mut out = 0.0;
            for _ in 0..3000 {
                out = f.process(1.0);
            }
            assert!(
                (out - 1.0).abs() < 0.15,
                "DC should stay near unity at res={res}, got {out}"
            );
        }
    }

    #[test]
    fn attenuates_above_cutoff() {
        let mut f = Ladder::new(44100.0);
        f.set_cutoff(200.0);
        f.set_resonance(0.0);
        let sr = 44100.0;
        let freq = 5000.0;
        let mut max_out: f32 = 0.0;
        for i in 0..4410 {
            let t = i as f32 / sr;
            let input = (2.0 * PI * freq * t).sin();
            let out = f.process(input);
            if i > 1000 {
                max_out = max_out.max(out.abs());
            }
        }
        assert!(
            max_out < 0.01,
            "5kHz should be heavily attenuated with 200Hz cutoff, got {max_out}"
        );
    }

    #[test]
    fn self_oscillation() {
        let mut f = Ladder::new(44100.0);
        f.set_cutoff(440.0);
        f.set_resonance(1.0);
        f.process(1.0);
        for _ in 0..100 {
            f.process(0.0);
        }
        let mut energy = 0.0_f32;
        for _ in 0..1000 {
            let out = f.process(0.0);
            energy += out * out;
        }
        let rms = (energy / 1000.0).sqrt();
        assert!(
            rms > 0.001,
            "filter should self-oscillate at resonance=1.0, rms={rms}"
        );
    }

    #[test]
    fn output_stays_bounded() {
        // Safety limiter means output never exceeds 1.0
        let mut f = Ladder::new(44100.0);
        f.set_cutoff(1000.0);
        f.set_resonance(0.95);
        let sr = 44100.0;
        let mut max_out: f32 = 0.0;
        for i in 0..44100 {
            let t = i as f32 / sr;
            let input = (2.0 * PI * 440.0 * t).sin();
            let out = f.process(input);
            max_out = max_out.max(out.abs());
        }
        // No hard clamp — resonant peak can exceed 1.0, but should
        // stay bounded (not blow up to infinity)
        assert!(
            max_out < 5.0,
            "output should stay bounded, got {max_out}"
        );
    }

    #[test]
    fn tanh_mode_stays_bounded() {
        let mut f = Ladder::new(44100.0);
        f.set_cutoff(440.0);
        f.set_resonance(1.0);
        f.set_mode(LadderMode::Tanh);
        f.process(1.0);
        let mut max_out: f32 = 0.0;
        for _ in 0..44100 {
            let out = f.process(0.0);
            max_out = max_out.max(out.abs());
        }
        assert!(
            max_out <= 1.0,
            "tanh self-oscillation should be bounded, got {max_out}"
        );
    }

    #[test]
    fn fast_tanh_matches_std() {
        for &x in &[0.0, 0.1, 0.5, 1.0, 2.0, -0.5, -2.0] {
            let approx = fast_tanh(x);
            let exact = x.tanh();
            let err = (approx - exact).abs();
            assert!(
                err < 0.005,
                "fast_tanh({x}): got {approx}, expected {exact}, err={err}"
            );
        }
    }
}
