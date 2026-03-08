use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dsp_lib::{Process, core::SharedParam, oscillator::Sine};
use std::io::*;


fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("Unable to get device.");
    let config = device.default_output_config()?;

    let sample_rate = config.sample_rate() as f32;
    let channels = config.channels() as usize;

    let shared_freq = SharedParam::new(440.0);
    let shared_freq_local = shared_freq.clone();

    let mut osc = Sine::new(sample_rate);
    osc.set_frequency(440.0);
    osc.set_amplitude(0.4);

    let stream = device.build_output_stream(
        &config.into(), 
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            osc.set_frequency(shared_freq.get());
            for frame in data.chunks_mut(channels) {
                let sample = osc.process();
                for ch in frame.iter_mut() {
                    *ch = sample;
                }
            }
        }, 
        move |err| eprintln!("Failed to build stream: {}", err), 
        None
    )?;

    stream.play()?;
    println!("Stream playing sine wave at 440 hz.");
    println!("Enter any frequency in Hz and press enter to change.");
    println!("Press q and enter to quit the program.");

    loop {
        print!("> ");
        stdout().flush()?;

        let mut input = String::new();
        stdin().read_line(&mut input)?;

        let trimmed = input.trim();
        if trimmed == "q" { break; }

        match trimmed.parse::<f32>() {
            Ok(freq) => {
                println!("-> {:.1} Hz", freq);
                shared_freq_local.set(freq);
            },
            Err( _) => {
                println!("Enter a number (freq in Hz) or press q to quit.");
            }
        }
    }

    Ok(())
}
