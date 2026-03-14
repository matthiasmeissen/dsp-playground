use crate::{Process, Reset, SetSampleRate, oscillator::Sine};


/// Low Frequency Oscillator — a slow sine used to modulate parameters.
///
/// Bipolar mode (default): output ranges from -depth to +depth.
///   Use for pitch modulation (vibrato): `base_freq + lfo.process()`
///
/// Unipolar mode: output ranges from 0.0 to depth.
///   Use for amplitude modulation (tremolo): `signal * lfo.process()`
///
/// Typical rates: 0.1–20 Hz. Above 20 Hz it becomes audible (FM territory).
pub struct Lfo {
    osc: Sine,
    depth: f32,
    unipolar: bool,
}

impl Lfo {
    pub fn new(sample_rate: f32) -> Self {
        Self { 
            osc: Sine::new(sample_rate), 
            depth: 1.0, 
            unipolar: false,
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.osc.set_frequency(freq);
    }

    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth;
    }

    pub fn set_unipolar(&mut self, is_unipolar: bool) {
        self.unipolar = is_unipolar;
    }
}

impl Process for Lfo {
    fn process(&mut self) -> f32 {
        let raw = self.osc.process();
        let shaped = if self.unipolar { 
            raw * 0.5 + 0.5 
        } else { 
            raw 
        };
        shaped * self.depth
    }
}

impl Reset for Lfo {
    fn reset(&mut self) {
        self.osc.reset();
    }
}

impl SetSampleRate for Lfo {
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.osc.set_sample_rate(sample_rate);
    }
}
