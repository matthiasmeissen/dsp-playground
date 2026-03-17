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
    voice_count: usize,
    base_frequency: f32,
    detune_cents: f32,
    amplitude: f32,
}

impl UnisonOscillator {
    pub fn new(sample_rate: f32, voice_count: usize) -> Self {
        let mut oscillators = Vec::new();
        for _ in 0..voice_count {
            oscillators.push(Sine::new(sample_rate));
        }
        Self {
            oscillators,
            voice_count,
            base_frequency: 440.0,
            detune_cents: 10.0,
            amplitude: 1.0,
        }
    }

    /// Sets the base frequency in Hz and redistributes all voices across the detune range.
    pub fn set_frequency(&mut self, freq: f32) {
        self.base_frequency = freq;

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

            let detuned_freq = self.base_frequency * cents_to_ratio(spread * self.detune_cents);
            self.oscillators[i].set_frequency(detuned_freq);
        }
    }

    /// Sets the output amplitude (1.0 = unity gain).
    pub fn set_amplitude(&mut self, amp: f32) {
        self.amplitude = amp;
    }

    /// Sets the total detune spread in cents and re-applies it to all voices.
    ///
    /// Voices are spread evenly from `-cents` to `+cents` around the base frequency.
    /// Common values: 5–30 for a classic unison thicken, up to 1200 for octave stacking.
    pub fn set_detune(&mut self, cents: f32) {
        self.detune_cents = cents;
        self.set_frequency(self.base_frequency);
    }
}

impl Process for UnisonOscillator {
    fn process(&mut self) -> f32 {
        let mut sum = 0.0;
        for oscillator in self.oscillators.iter_mut() {
            sum += oscillator.process()
        }
        // Divide by voice count to keep output level constant regardless of how
        // many voices are active (averaging instead of summing prevents clipping).
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
