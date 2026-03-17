use std::f32::consts::TAU;

use crate::{Process, Reset, SetSampleRate};


/// A sine oscillator that computes each sample with `sin()`.
///
/// Produces a pure, alias-free sine wave at the cost of one `sin()` call per
/// sample. For many simultaneous voices, `Wavetable` is cheaper because it
/// approximates the same waveform via a precomputed lookup table.
pub struct Sine {
    phase: f32,
    phase_increment: f32,
    sample_rate: f32,
    amplitude: f32,
}

impl Sine {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            phase: 0.0,
            phase_increment: 0.0,
            sample_rate,
            amplitude: 1.0,
        }
    }

    /// Sets the oscillator frequency in Hz.
    pub fn set_frequency(&mut self, freq: f32) {
        self.phase_increment = freq / self.sample_rate;
    }

    /// Sets the output amplitude (1.0 = unity gain).
    pub fn set_amplitude(&mut self, amp: f32) {
        self.amplitude = amp;
    }
}

impl Process for Sine {
    fn process(&mut self) -> f32 {
        let sample = (self.phase * TAU).sin();
        
        self.phase += self.phase_increment;
        if self.phase >= 1.0 { self.phase -= 1.0 };

        sample * self.amplitude
    }
}

impl Reset for Sine {
    fn reset(&mut self) {
        self.phase = 0.0;
    }
}

impl SetSampleRate for Sine {
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}
