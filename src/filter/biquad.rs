
use crate::{Transform, Reset, SetSampleRate};

/// Filter mode selection for the Biquad.
pub enum BiquadMode {
    Lowpass,
    Highpass,
    Bandpass,
}

/// Two-pole biquad filter with configurable mode (lowpass, highpass, bandpass).
///
/// Steeper than a one-pole (12 dB/octave vs 6 dB/octave) and supports resonance (Q).
/// Coefficients are from the Audio EQ Cookbook (Robert Bristow-Johnson).
///
/// # Usage
/// ```
/// use dsp_lib::{Transform, filter::biquad::{Biquad, BiquadMode}};
///
/// let mut filter = Biquad::new(44100.0);
/// filter.set_params(800.0, 0.707, BiquadMode::Lowpass);
/// let output = filter.process(1.0);
/// ```
pub struct Biquad {
    // Coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    // State - past inputs and outputs
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    // Parameters
    sample_rate: f32,
}

impl Biquad {
    pub fn new(sample_rate: f32) -> Self {
        Self { 
            b0: 0.0, b1: 0.0, b2: 0.0, 
            a1: 0.0, a2: 0.0, 
            x1: 0.0, x2: 0.0, 
            y1: 0.0, y2: 0.0, 
            sample_rate 
        }
    }

    /// Set filter cutoff (Hz), resonance (Q), and mode.
    ///
    /// - `cutoff`: frequency in Hz where the filter acts
    /// - `q`: resonance. 0.707 = flat (Butterworth), 1.0 = mild peak, 2.0–5.0 = audible resonance, 10.0+ = sharp ringing
    /// - `mode`: Lowpass, Highpass, or Bandpass
    pub fn set_params(&mut self, cutoff: f32, q: f32, mode: BiquadMode) {
        let w0 = std::f32::consts::TAU * cutoff / self.sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);

        let (b0, b1, b2, a0, a1, a2) = match mode {
            BiquadMode::Lowpass => {
                let b1 = 1.0 - cos_w0;
                let b0 = b1 / 2.0;
                let b2 = b0;
                (b0, b1, b2, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha)
            }
            BiquadMode::Highpass => {
                let b1 = -(1.0 + cos_w0);
                let b0 = -b1 / 2.0;
                let b2 = b0;
                (b0, b1, b2, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha)
            }
            BiquadMode::Bandpass => {
                let b0 = alpha;
                let b2 = -alpha;
                (b0, 0.0, b2, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha)
            }
        };

        // Normalize by a0
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }
}

impl Transform for Biquad {
    fn process(&mut self, input: f32) -> f32 {
        let mut y = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
                    - self.a1 * self.y1 - self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;

        if y.abs() < 1e-15 { y = 0.0; }
        self.y1 = y;

        y
    }
}

impl Reset for Biquad {
    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

impl SetSampleRate for Biquad {
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}
