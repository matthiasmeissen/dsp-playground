use crate::{Reset, SetSampleRate, Transform};

/// One-pole lowpass filter (6 dB/octave).
///
/// The simplest useful filter: smooths a signal by blending each sample
/// with the previous output. Also used for parameter smoothing to prevent
/// zipper noise on modulated controls.
///
/// # Usage
/// ```
/// use dsp_lib::{Transform, filter::one_pole::OnePole};
///
/// let mut filter = OnePole::new(44100.0);
/// filter.set_cutoff(800.0);
/// let output = filter.process(1.0);
/// ```
pub struct OnePole {
    state: f32,
    coefficient: f32,
    sample_rate: f32,
}

impl OnePole {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            state: 0.0,
            coefficient: 0.0,
            sample_rate,
        }
    }

    /// Set filter cutoff frequency in Hz.
    ///
    /// Higher cutoff = less filtering (signal follows input more closely).
    /// Lower cutoff = more smoothing. For parameter smoothing, 50–200 Hz
    /// is typical (~5–20 ms response time).
    pub fn set_cutoff(&mut self, cutoff: f32) {
        let rc = 1.0 / (std::f32::consts::TAU * cutoff);
        let dt = 1.0 / self.sample_rate;
        self.coefficient = dt / (rc + dt);
    }

    /// Highpass: returns `input - lowpass(input)`.
    ///
    /// Subtracting the smooth (low frequency) part leaves only the sharp
    /// (high frequency) part. Use with a low cutoff (~10 Hz) for DC blocking.
    pub fn process_highpass(&mut self, input: f32) -> f32 {
        let lp = self.process(input);
        input - lp
    }

    /// Create a DC blocker — a highpass at 10 Hz.
    ///
    /// Removes constant offset from a signal without affecting audible content.
    /// Use with `process_highpass()`:
    /// ```
    /// use dsp_lib::filter::one_pole::OnePole;
    ///
    /// let mut dc = OnePole::dc_blocker(44100.0);
    /// let clean = dc.process_highpass(1.5); // removes the DC offset
    /// ```
    pub fn dc_blocker(sample_rate: f32) -> Self {
        let mut f = Self::new(sample_rate);
        f.set_cutoff(10.0);
        f
    }
}

impl Transform for OnePole {
    fn process(&mut self, input: f32) -> f32 {
        self.state += self.coefficient * (input - self.state);
        self.state
    }
}

impl Reset for OnePole {
    fn reset(&mut self) {
        self.state = 0.0;
    }
}

impl SetSampleRate for OnePole {
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}
