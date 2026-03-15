use crate::{Process, Reset, SetSampleRate, oscillator::Sine};


fn cents_to_ratio(cents: f32) -> f32 {
    //(cents / 1200.0).exp2()
    2.0f32.powf(cents / 1200.0)
}

/// Creates multiple sin oscillators that can be detuned
/// Use cents (1200 is one octave)
/// Example: 
/// 5-30 cents creates unison
/// 200 chord (1.1125 two semitones up)
/// 700 chord (1.5 perfect fifth)
pub struct UnisonOscillator {
    oscillators: Vec<Sine>, // Not applicable in no_std
    amplitude: f32,
    detune_cents: f32,
    base_freqency: f32,
    voice_count: usize,
}

impl UnisonOscillator {
    pub fn new(sample_rate: f32, voice_count: usize) -> Self {
        let mut oscillators = Vec::new();
        for _ in 0..voice_count {
            oscillators.push(Sine::new(sample_rate));
        }
        Self { 
            oscillators, 
            amplitude: 1.0, 
            detune_cents: 10.0, 
            base_freqency: 440.0, 
            voice_count,
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.base_freqency = freq;

        for i in 0..self.voice_count {
            let spread = if self.voice_count > 1 {
                // (0 / 4) * 2.0 - 1.0 => -1
                // (1 / 4) * 2.0 - 1.0 => -0.5
                // (2 / 4) * 2.0 - 1.0 => 0
                // (3 / 4) * 2.0 - 1.0 => 0.5
                // (4 / 4) * 2.0 - 1.0 => 1
                (i as f32 / (self.voice_count - 1) as f32) * 2.0 - 1.0
            } else {
                0.0
            };

            let detuned_freq = self.base_freqency * cents_to_ratio(spread * self.detune_cents);
            self.oscillators[i].set_frequency(detuned_freq);
        }
    }

    pub fn set_detune(&mut self, cents: f32) {
        self.detune_cents = cents;
        self.set_frequency(self.base_freqency);
    }

    pub fn set_amplitude(&mut self, amp: f32) {
        self.amplitude = amp;
    }
}

impl Process for UnisonOscillator {
    fn process(&mut self) -> f32 {
        let mut sum = 0.0;
        for oscillator in self.oscillators.iter_mut() {
            sum += oscillator.process()
        }
        (sum / self.voice_count as f32) * self.amplitude
    }
}

impl Reset for UnisonOscillator {
    fn reset(&mut self) {
        for oscillator in self.oscillators.iter_mut() {
            oscillator.reset();
        }
    }
}

impl SetSampleRate for UnisonOscillator {
    fn set_sample_rate(&mut self, sample_rate: f32) {
        for oscillator in self.oscillators.iter_mut() {
            oscillator.set_sample_rate(sample_rate);
        }
    }
}
