use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dsp_lib::{Process, oscillator::Sine};

mod utils { pub mod render; }
use crate::utils::render::render_to_wav;

struct FmSynth {
    carrier: Sine,
    modulator: Sine,
    carrier_freq: f32,
    mod_freq: f32,
    mod_depth: f32,
}

impl FmSynth {
    fn vibrato(sample_rate: f32) -> Self {
        Self { 
            carrier: Sine::new(sample_rate), 
            modulator: Sine::new(sample_rate),
            carrier_freq: 440.0, 
            mod_freq: 20.0, 
            mod_depth: 20.0 
        }
    }

    fn bell(sample_rate: f32) -> Self {
        Self { 
            carrier: Sine::new(sample_rate), 
            modulator: Sine::new(sample_rate),
            carrier_freq: 440.0, 
            mod_freq: 200.0, 
            mod_depth: 500.0 
        }
    }
}

impl Process for FmSynth {
    fn process(&mut self) -> f32 {
        self.modulator.set_frequency(self.mod_freq);
        self.carrier.set_amplitude(0.4);

        let modulator_with_depth = self.modulator.process() * self.mod_depth;
        self.carrier.set_frequency(self.carrier_freq + modulator_with_depth);
        self.carrier.process()
    }
}

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("Failed to create outout device.");
    let config = device.default_output_config()?;

    let sample_rate = config.sample_rate() as f32;
    let channels = config.channels() as usize;

    //let fm_synth_vibrato = FmSynth::vibrato(sample_rate);
    let mut fm_synth_bell = FmSynth::bell(sample_rate);

    render_to_wav("./output/fm_bell.wav", sample_rate, 2.0, || fm_synth_bell.process(), true);

    let stream = device.build_output_stream(
        &config.into(), 
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {

                let sample = fm_synth_bell.process();

                for ch in frame.iter_mut() {
                    *ch = sample;
                }
            }
        }, 
        |err| eprintln!("Failed building the stream: {}", err), 
        None
    )?;

    stream.play()?;
    println!("Stream is playing. Press enter to stop.");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}
