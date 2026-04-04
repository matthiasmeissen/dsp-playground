use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use dsp_lib::{Process, Transform};
use dsp_lib::core::SharedParam;
use dsp_lib::oscillator::{BlepOscillator, Waveform, Lfo};
use dsp_lib::filter::{svf::*, ladder::*};


fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("Failed to create device.");
    let config = device.default_output_config()?;

    let sample_rate = config.sample_rate() as f32;
    let channels = config.channels() as usize;

    // Synth Elements
    let mut osc = BlepOscillator::new(sample_rate, Waveform::Saw);
    let mut lfo = Lfo::new(sample_rate);
    lfo.set_unipolar(true);
    let mut svf_filter = Svf::new(sample_rate);
    let mut ladder_filter = Ladder::new(sample_rate);

    // Synth Parameters
    let main_freq = SharedParam::new(440.0);
    let loop_freq = main_freq.clone();
    let main_cutoff = SharedParam::new(200.0);
    let loop_cutoff = main_cutoff.clone();
    let main_use_ladder = Arc::new(AtomicBool::new(false));
    let loop_use_ladder = main_use_ladder.clone();

    let stream = device.build_output_stream(
        &config.into(), 
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                osc.set_frequency(loop_freq.get());
                lfo.set_frequency(4.0);
                let cutoff = loop_cutoff.get() + lfo.process() * 800.0;
                svf_filter.set_params(cutoff, 2.0);
                ladder_filter.set_cutoff(cutoff);
                ladder_filter.set_resonance(0.6);
                let sample = match loop_use_ladder.load(Ordering::Relaxed) {
                    true => ladder_filter.process(osc.process()),
                    false => svf_filter.process(osc.process()),
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

    println!("Playing a polyblep saw at 220Hz filtered through SVF Filter. Press q to stop.");
    
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();

        if let Some(rest) = trimmed.strip_prefix("c ") {
            if let Ok(value) = rest.parse::<f32>() {
                main_cutoff.set(value);
                println!("Cutoff: {} Hz", value);
            }
        } else if trimmed == "l" {
            main_use_ladder.store(true, Ordering::Relaxed);
            println!("Filter: Ladder");
        } else if trimmed == "s" {
            main_use_ladder.store(false, Ordering::Relaxed);
            println!("Filter: State Variable");
        } else if trimmed == "q" {
            break;
        }
    }

    Ok(())
}
