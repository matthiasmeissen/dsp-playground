use cpal::traits::{DeviceTrait, HostTrait};
use dsp_lib::{Process, oscillator::{Naive, Waveform}};

mod utils { pub mod render; }
use crate::utils::render::render_to_wav;


fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("Failed to create output device.");
    let config = device.default_output_config()?;

    let sample_rate = config.sample_rate() as f32;
    let channels = config.channels() as usize;

    let mut osc = Naive::new(sample_rate, Waveform::Saw);
    osc.set_frequency(200.0);
    osc.set_amplitude(0.4);

    render_to_wav("./output/naive_saw_200.wav", sample_rate, 2.0, || osc.process(), true);

    // let stream = device.build_output_stream(
    //     &config.into(), 
    //     move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
    //         for frame in data.chunks_mut(channels) {
    //             let sample = osc.process();
    //             for ch in frame.iter_mut() {
    //                 *ch = sample;
    //             }
    //         }
    //     }, 
    //     move |err| eprintln!("Failed to create output stream: {}", err), 
    //     None
    // )?;

    // stream.play()?;

    // println!("Playing a naive oscillator at 440Hz. Press enter to stop.");
    // let mut input = String::new();
    // std::io::stdin().read_line(&mut input)?;

    Ok(())
}
