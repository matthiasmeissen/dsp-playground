use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use dsp_lib::{Process, Transform, filter::biquad::*, oscillator::{BlepOscillator, Waveform, Lfo}};


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

    let mut filter = Biquad::new(sample_rate);
    filter.set_params(200.0, 0.8, BiquadMode::Lowpass);

    let stream = device.build_output_stream(
        &config.into(), 
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let cutoff = 100.0 + lfo.process();
                filter.set_params(cutoff, 4.0, BiquadMode::Lowpass);
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

    println!("Playing a filtered polyblep saw at 220Hz with modulated cutoff. Press enter to stop.");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}
