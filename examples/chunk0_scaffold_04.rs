mod utils { pub mod render; }
use crate::utils::render::render_to_wav;

use dsp_lib::{Process, oscillator::Sine};

fn main() {
    let sample_rate: f32 = 44100.0;

    let mut osc = Sine::new(sample_rate);
    osc.set_frequency(440.0);
    osc.set_amplitude(0.4);

    render_to_wav("./output/sine_440.wav", sample_rate, 2.0, || osc.process(), true);
}