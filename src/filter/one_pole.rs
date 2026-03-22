use crate::{Reset, SetSampleRate, Transform};


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
            sample_rate
        }
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        let rc = 1.0 / (std::f32::consts::TAU * cutoff);
        let dt = 1.0 / self.sample_rate;
        self.coefficient = dt / (rc + dt);
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
        self.sample_rate = sample_rate
    }
}
