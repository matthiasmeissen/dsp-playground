use crate::{Reset, SetSampleRate, Transform};


/// State Variable Filter — gives lowpass, bandpass, and highpass simultaneously.
///
/// More stable at high resonance than a biquad, and cheaper to modulate since
/// parameter updates don't require full coefficient recalculation.
///
/// # Usage
/// ```
/// use dsp_lib::filter::svf::Svf;
///
/// let mut svf = Svf::new(44100.0);
/// svf.set_params(800.0, 2.0);  // cutoff 800 Hz, moderate resonance
/// let (lp, bp, hp) = svf.process_all(1.0);
/// ```
pub struct Svf {
    lp: f32,
    bp: f32,
    f: f32,
    q: f32,
    sample_rate: f32,
}

impl Svf {
    pub fn new(sample_rate: f32) -> Self {
        Self { 
            lp: 0.0, 
            bp: 0.0, 
            f: 0.0, 
            q: 1.0, 
            sample_rate 
        }
    }

    /// Set filter cutoff (Hz) and resonance.
    ///
    /// - `cutoff`: frequency in Hz where the filter acts
    /// - `resonance`: 0.707 = flat (Butterworth), 2.0–5.0 = audible peak, 10.0+ = sharp ringing.
    ///   Internally inverted (1/resonance), so higher values = more resonance.
    pub fn set_params(&mut self, cutoff: f32, resonance: f32) {
        self.f = 2.0 * (std::f32::consts::PI * cutoff / self.sample_rate).sin();
        self.q = 1.0 / resonance;
    }

    /// Returns (lowpass, bandpass, highpass)
    pub fn process_all(&mut self, input: f32) -> (f32, f32, f32) {
        let hp = input - self.lp - self.q * self.bp;
        self.bp += self.f * hp;
        self.lp += self.f * self.bp;

        if self.bp.abs() < 1e-15 { self.bp = 0.0; }
        if self.lp.abs() < 1e-15 { self.lp = 0.0; }

        (self.lp, self.bp, hp)
    }
}

impl Transform for Svf {
    fn process(&mut self, input: f32) -> f32 {
        let (lp, _, _) = self.process_all(input);
        lp
    }
}

impl Reset for Svf {
    fn reset(&mut self) {
        self.lp = 0.0;
        self.bp = 0.0;
    }
}

impl SetSampleRate for Svf {
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}
