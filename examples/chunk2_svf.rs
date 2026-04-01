use std::sync::{Arc, atomic::{AtomicU8, Ordering}};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use dsp_lib::{Process, Transform};
use dsp_lib::core::SharedParam;
use dsp_lib::oscillator::{BlepOscillator, Waveform, Lfo};
use dsp_lib::filter::svf::*;


fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("Failed to create device.");
    let config = device.default_output_config()?;

    let sample_rate = config.sample_rate() as f32;
    let channels = config.channels() as usize;

    let mut osc = BlepOscillator::new(sample_rate, Waveform::Saw);
    osc.set_frequency(220.0);
    osc.set_amplitude(0.4);

    let mut lfo = Lfo::new(sample_rate);
    lfo.set_frequency(2.0);
    lfo.set_unipolar(true);
    lfo.set_depth(1800.0);

    let main_cutoff = SharedParam::new(200.0);
    let shared_cutoff = main_cutoff.clone();

    let main_res = SharedParam::new(0.707);
    let shared_res = main_res.clone();

    let filter_type = Arc::new(AtomicU8::new(0));
    let shared_filter_type = filter_type.clone();

    let mut filter = Svf::new(sample_rate);
    filter.set_params(main_cutoff.get(), main_res.get());

    let stream = device.build_output_stream(
        &config.into(), 
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let cutoff = shared_cutoff.get() + lfo.process();
                filter.set_params(cutoff, shared_res.get());
                let filter_output = filter.process_all(osc.process());
                let sample = match shared_filter_type.load(Ordering::Relaxed) {
                    0 => filter_output.0,
                    1 => filter_output.1,
                    2 => filter_output.2,
                    _ => filter_output.0,
                };
                for ch in frame.iter_mut() {
                    *ch = sample;
                }
            }
        }, 
        move |err| eprintln!("Failed to create output stream: {}", err), 
        None
    )?;

    stream.play()?;

    println!("Playing a filtered polyblep saw at 220Hz with modulated cutoff. Press q to stop.");
    
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();

        if let Ok(value) = trimmed.parse::<f32>() {
            let clamped_res = value.clamp(0.0, 20.0);
            main_res.set(clamped_res);
            println!("Resonance: {}", clamped_res);
        }

        if let Some(rest) = trimmed.strip_prefix("c ") {
            if let Ok(value) = rest.parse::<f32>() {
                main_cutoff.set(value);
                println!("Cutoff: {} Hz", value);
            }
        } else if trimmed == "l" {
            filter_type.store(0, Ordering::Relaxed);
            println!("Filter Type set to Lowpass");
        } else if trimmed == "b" {
            filter_type.store(1, Ordering::Relaxed);
            println!("Filter Type set to Bandpass");
        } else if trimmed == "h" {
            filter_type.store(2, Ordering::Relaxed);
            println!("Filter Type set to Hightpass");
        } else if trimmed == "q" {
            break;
        }
    }

    Ok(())
}
