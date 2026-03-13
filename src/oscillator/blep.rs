use crate::{Process, Reset, SetSampleRate, oscillator::Waveform};


/// Helper function that checks if it is within one sample of an edge
/// Returns a small correction value
fn poly_blep(phase: f32, increment: f32) -> f32 {
    if phase < increment {
        // Just after a discontinuity (example: saw 1.0 to 0.0)
        let t = phase / increment;
        2.0 * t - t * t - 1.0
    } else if phase > 1.0 - increment {
        // Just before a discontinuity (example: square 0.0 to 1.0)
        let t = (phase - 1.0) / increment;
        t * t + 2.0 * t + 1.0
    } else {
        0.0
    }
}

/// Oscillator with antialiasing
/// waveform: saw, square, triangle
pub struct BlepOscillator {
    phase: f32,
    phase_increment: f32,
    amplitude: f32,
    sample_rate: f32,
    waveform: Waveform,
}

impl BlepOscillator {
    pub fn new(sample_rate: f32, waveform: Waveform) -> Self {
        Self { phase: 0.0, phase_increment: 0.0, amplitude: 1.0, sample_rate, waveform }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.phase_increment = freq / self.sample_rate;
    }

    pub fn set_amplitude(&mut self, amp: f32) {
        self.amplitude = amp;
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }
}

impl Process for BlepOscillator {
    fn process(&mut self) -> f32 {
        let out = match self.waveform {
            Waveform::Saw => {
                let naive_saw = self.phase * 2.0 - 1.0;
                naive_saw - poly_blep(self.phase, self.phase_increment)
            },
            Waveform::Square => {
                let naive_square = if self.phase < 0.5 { 1.0 } else { -1.0 };
    
                let mut half_phase = self.phase + 0.5;
                if half_phase >= 1.0 { half_phase -= 1.0; }

                naive_square 
                    + poly_blep(self.phase, self.phase_increment)
                    - poly_blep(half_phase, self.phase_increment)
            },
            Waveform::Triangle => {
                if self.phase < 0.5 {
                    self.phase * 4.0 - 1.0
                } else {
                    3.0 - self.phase * 4.0
                }
            }
        };

        self.phase += self.phase_increment;
        if self.phase >= 1.0 { self.phase -= 1.0 };
        
        out * self.amplitude
    }
}

impl Reset for BlepOscillator {
    fn reset(&mut self) {
        self.phase = 0.0;
    }
}

impl SetSampleRate for BlepOscillator {
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}

