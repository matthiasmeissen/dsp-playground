/// A module that generates one audio sample at a time.
/// Generators (oscillators, noise) implement this.
pub trait Process {
    fn process(&mut self) -> f32;
}

/// A module that transforms one sample at a time.
/// Processors (filters, effects) implement this.
/// Takes input audio, returns processed audio.
pub trait Transform {
    fn process(&mut self, input: f32) -> f32;
}

/// Anything that can be reset to its initial state.
/// Examples are (Phase, Buffer, Envelope).
pub trait Reset {
    fn reset(&mut self);
}

/// Anything that needs to know the sample rate to compute.
pub trait SetSampleRate {
    fn set_sample_rate(&mut self, sample_rate: f32);
}