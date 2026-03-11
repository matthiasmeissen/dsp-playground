
mod utils { pub mod render; }
use dsp_lib::{Process, oscillator::Wavetable};

use crate::utils::render::render_to_wav;

fn main() -> anyhow::Result<()> {
    let sample_rate = 44100.0;

    let mut osc = Wavetable::new_sine(sample_rate);
    osc.set_frequency(440.0);
    osc.set_amplitude(0.4);

    render_to_wav("./output/wavetable_sin_440.wav", sample_rate, 2.0, || osc.process(), true);

    Ok(())
}
