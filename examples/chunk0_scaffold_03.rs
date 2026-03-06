use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dsp_lib::{Process, oscillator::Sine};



fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device found.");
    let config = device.default_output_config()?;

    let sample_rate = config.sample_rate() as f32;
    let channels = config.channels() as usize;

    let mut osc = Sine::new(sample_rate);
    osc.set_frequency(440.0);
    osc.set_amplitude(0.3);

    let mut osc2 = Sine::new(sample_rate);
    osc2.set_frequency(440.0 * 1.25);
    osc2.set_amplitude(0.3);

    let stream = device.build_output_stream(
        &config.into(), 
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let sample = (osc.process() + osc2.process()) * 0.4;
                for ch in frame.iter_mut() {
                    *ch = sample;
                }
            }
        }, 
        move |err| eprintln!("Error building the stream: {}", err), 
        None
    )?;

    stream.play()?;
    println!("Playing sine wave from library. Press enter to stop.");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}
