use crate::{Process, Reset, SetSampleRate};


pub enum Waveform {
    Saw,
    Square,
    Triangle,
}

/// Simple oscillator using direct phase-to-waveform math.
/// 
/// Produces saw, square, and triangle from the same phase accumulator.
/// "Naive" because sharp waveform edges cause aliasing at higher frequencies.
/// Use other oscillators instead when aliasing matters (above ~1kHz).
pub struct Naive {
    phase: f32,
    phase_increment: f32,
    sample_rate: f32,
    amplitude: f32,
    waveform: Waveform,
}

impl Naive {
    pub fn new(sample_rate: f32, waveform: Waveform) -> Self {
        Self { 
            phase: 0.0, 
            phase_increment: 0.0, 
            sample_rate, 
            amplitude: 1.0, 
            waveform
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

    /// Switches to a different waveform without resetting the phase.
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }
}

impl Process for Naive {
    fn process(&mut self) -> f32 {
        let sample = match self.waveform {
            // Phase [0, 1) mapped linearly to [-1, +1).
            Waveform::Saw => {
                self.phase * 2.0 - 1.0
            },
            // High for the first half-cycle, low for the second.
            Waveform::Square => {
                if self.phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            },
            // Rises from -1 to +1 in the first half, falls back to -1 in the second.
            // Derived by folding a saw: abs(saw) * 2 - 1, written as two linear segments.
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
        
        sample * self.amplitude
    }
}

impl Reset for Naive {
    fn reset(&mut self) {
        self.phase = 0.0;
    }
}

impl SetSampleRate for Naive {
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}
