use std::f32::consts::TAU;

use crate::{Process, Reset, SetSampleRate};

const TABLE_SIZE: usize = 2048;

/// Wavetable Oscialltor using precomputed waves
pub struct Wavetable {
    table: [f32; TABLE_SIZE],
    phase: f32,
    phase_increment: f32,
    sample_rate: f32,
    amplitude: f32,
}

impl Wavetable {
    pub fn new_sine(sample_rate: f32) -> Self {
        let mut table = [0.0; TABLE_SIZE];
        for i in 0..TABLE_SIZE {
            table[i] = (i as f32 / TABLE_SIZE as f32 * TAU).sin();
        }
        Self { 
            table, 
            phase: 0.0, 
            phase_increment: 0.0, 
            sample_rate, 
            amplitude: 1.0, 
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.phase_increment = freq / self.sample_rate;
    }

    pub fn set_amplitude(&mut self, amp: f32) {
        self.amplitude = amp;
    }
}

impl Process for Wavetable {
    fn process(&mut self) -> f32 {
        // Linear Interpolation
        // Convert phase to fractional table index (example: 1248.34)
        let idx = self.phase * TABLE_SIZE as f32;
        // Get current integer index and next wrapping one (example: 1248 and 1249)
        let idx0 = idx as usize % TABLE_SIZE;
        let idx1 = (idx0 + 1) % TABLE_SIZE;
        // Get fractional part (example: 0.34)
        let frac = idx - idx.floor();

        // Generate output value by blending current and next index together
        let out = self.table[idx0] * (1.0 - frac) + self.table[idx1] * frac;

        // Wrap phase to go from 0.0 - 1.0
        self.phase = (self.phase + self.phase_increment) % 1.0;
        out
    }
}

impl Reset for Wavetable {
    fn reset(&mut self) {
        self.phase = 0.0;
    }
}

impl SetSampleRate for Wavetable {
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}
