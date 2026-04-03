use std::sync::{Arc, atomic::{AtomicBool, AtomicU8, Ordering}};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use dsp_lib::{Process, Transform, filter::one_pole::OnePole, oscillator::Naive};
use dsp_lib::core::SharedParam;
use dsp_lib::oscillator::{BlepOscillator, Waveform, Lfo};
use dsp_lib::filter::svf::*;


fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("Failed to create device.");
    let config = device.default_output_config()?;

    let sample_rate = config.sample_rate() as f32;
    let channels = config.channels() as usize;

    // Synth Elements
    let mut osc = BlepOscillator::new(sample_rate, Waveform::Saw);
    let mut lfo = Naive::new(sample_rate, Waveform::Square);
    let mut filter = Svf::new(sample_rate);
    let mut signal_smoother = OnePole::new(sample_rate);
    signal_smoother.set_cutoff(50.0); //50Hz smoothing -> 20ms response

    // Synth Parameters
    let main_freq = SharedParam::new(440.0);
    let loop_freq = main_freq.clone();
    let main_cutoff = SharedParam::new(200.0);
    let loop_cutoff = main_cutoff.clone();
    let main_is_smoothing = Arc::new(AtomicBool::new(false));
    let loop_is_smoothing = main_is_smoothing.clone();

    let stream = device.build_output_stream(
        &config.into(), 
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                osc.set_frequency(loop_freq.get());
                lfo.set_frequency(10.0);
                let raw_cutoff = loop_cutoff.get() + (lfo.process() + 1.0) * 800.0;
                let cutoff = match loop_is_smoothing.load(Ordering::Relaxed) {
                    true => signal_smoother.process(raw_cutoff),
                    false => raw_cutoff,
                };
                filter.set_params(cutoff, 2.0);
                let sample = filter.process(osc.process());
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

        if let Some(rest) = trimmed.strip_prefix("c ") {
            if let Ok(value) = rest.parse::<f32>() {
                main_cutoff.set(value);
                println!("Cutoff: {} Hz", value);
            }
        } else if trimmed == "t" {
            main_is_smoothing.store(true, Ordering::Relaxed);
            println!("Smoothing is on.");
        } else if trimmed == "f" {
            main_is_smoothing.store(false, Ordering::Relaxed);
            println!("Smoothing is off.");
        } else if trimmed == "q" {
            break;
        }
    }

    Ok(())
}
