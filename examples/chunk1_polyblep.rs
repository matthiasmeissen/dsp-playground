
mod utils { pub mod render; }
use dsp_lib::{Process, oscillator::{BlepOscillator, Naive, Waveform}};

use crate::utils::render::render_to_wav;

fn main() -> anyhow::Result<()> {
    let sample_rate = 44100.0;

    // Compare Saw Waves

    let mut naive_saw = Naive::new(sample_rate, Waveform::Saw);
    naive_saw.set_frequency(4000.0);
    naive_saw.set_amplitude(0.4);

    let mut polyblep_saw = BlepOscillator::new(sample_rate, Waveform::Saw);
    polyblep_saw.set_frequency(2000.0);
    polyblep_saw.set_amplitude(0.4);

    //render_to_wav("./output/naive_saw_4000.wav", sample_rate, 2.0, || naive_saw.process(), true);
    render_to_wav("./output/polyblep_saw_2000.wav", sample_rate, 2.0, || polyblep_saw.process(), true);

    // Compare Square Waves

    let mut naive_square = Naive::new(sample_rate, Waveform::Square);
    naive_square.set_frequency(4000.0);
    naive_square.set_amplitude(0.4);

    let mut polyblep_square = BlepOscillator::new(sample_rate, Waveform::Square);
    polyblep_square.set_frequency(2000.0);
    polyblep_square.set_amplitude(0.4);

    //render_to_wav("./output/naive_square_4000.wav", sample_rate, 2.0, || naive_square.process(), true);
    render_to_wav("./output/polyblep_square_2000.wav", sample_rate, 2.0, || polyblep_square.process(), true);

    Ok(())
}
