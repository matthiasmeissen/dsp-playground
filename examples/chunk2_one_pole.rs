use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dsp_lib::{Process, Transform, core::SharedParam, filter::one_pole::OnePole, oscillator::*};


fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("Failed to create device.");
    let config = device.default_output_config()?;

    let sample_rate = config.sample_rate() as f32;
    let channels = config.channels() as usize;

    let main_cutoff = SharedParam::new(200.0);
    let shared_cutoff = main_cutoff.clone();

    let mut osc = BlepOscillator::new(sample_rate, Waveform::Saw);
    osc.set_frequency(220.0);
    osc.set_amplitude(0.4);

    let mut filter = OnePole::new(sample_rate);
    filter.set_cutoff(200.0);

    let stream = device.build_output_stream(
        &config.into(), 
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            filter.set_cutoff(shared_cutoff.get());
            for frame in data.chunks_mut(channels) {
                let osc_signal = osc.process();
                let sample = filter.process(osc_signal);
                for ch in frame.iter_mut() {
                    *ch = sample;
                }
            }
        }, 
        move |err| eprintln!("Failed to create output stream: {}", err), 
        None
    )?;

    stream.play()?;

    println!("Playing a filtered saw wave. Enter a freq. Press q to stop.");

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();

        if trimmed == "q" {
            break;
        }

        if let Ok(value) = trimmed.parse::<f32>() {
            main_cutoff.set(value);
            println!("Cutoff: {} Hz", value);
        }
    }

    Ok(())
}
