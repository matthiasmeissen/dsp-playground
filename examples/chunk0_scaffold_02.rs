use std::f32::consts::TAU;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};


fn main() -> anyhow::Result<()> {
    // Setup basic audio device
    let host = cpal::default_host();
    let device = host.default_output_device().expect("NO output device found.");
    let config = device.default_output_config()?;

    // Extract sample rate and channel number
    let sample_rate = config.sample_rate() as f32;
    let channels = config.channels() as usize;

    // Define global state that lives across callbacks
    let mut phase: f32 = 0.0;
    let freq: f32 = 440.0;
    let phase_increment = freq / sample_rate;

    // Setup audio stream
    let stream = device.build_output_stream(
        &config.into(), 
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let sample = (phase * TAU).sin() * 0.3;
                phase = (phase + phase_increment) % 1.0;
                for ch in frame.iter_mut() {
                    *ch = sample;
                }
            }
        }, 
        move |err| eprintln!("Stream error: {}", err), 
        None
    )?;

    // Play the stream
    stream.play()?;
    println!("Stream playing sine wave at 440hz. Press enter to stop.");

    // Wait for input to keep thread running
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}
