use std::sync::{Arc, Mutex};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dsp_lib::{
    Process,
    oscillator::{
        Sine, Naive, Wavetable, BlepOscillator, UnisonOscillator, Waveform,
    },
};

// Holds one instance of every oscillator so the audio callback can switch
// between them without allocation on the hot path.
struct Oscillators {
    sine:          Sine,
    naive_saw:     Naive,
    naive_square:  Naive,
    naive_tri:     Naive,
    blep_saw:      BlepOscillator,
    blep_square:   BlepOscillator,
    wavetable:     Wavetable,
    unison:        UnisonOscillator,
}

impl Oscillators {
    fn new(sample_rate: f32) -> Self {
        let freq = 440.0;
        let amp  = 0.4;

        let mut sine = Sine::new(sample_rate);
        sine.set_frequency(freq);
        sine.set_amplitude(amp);

        let mut naive_saw = Naive::new(sample_rate, Waveform::Saw);
        naive_saw.set_frequency(freq);
        naive_saw.set_amplitude(amp);

        let mut naive_square = Naive::new(sample_rate, Waveform::Square);
        naive_square.set_frequency(freq);
        naive_square.set_amplitude(amp);

        let mut naive_tri = Naive::new(sample_rate, Waveform::Triangle);
        naive_tri.set_frequency(freq);
        naive_tri.set_amplitude(amp);

        let mut blep_saw = BlepOscillator::new(sample_rate, Waveform::Saw);
        blep_saw.set_frequency(freq);
        blep_saw.set_amplitude(amp);

        let mut blep_square = BlepOscillator::new(sample_rate, Waveform::Square);
        blep_square.set_frequency(freq);
        blep_square.set_amplitude(amp);

        let mut wavetable = Wavetable::new_sine(sample_rate);
        wavetable.set_frequency(freq);
        wavetable.set_amplitude(amp);

        let mut unison = UnisonOscillator::new(sample_rate, 5);
        unison.set_frequency(freq);
        unison.set_detune(15.0);
        unison.set_amplitude(amp);

        Self { sine, naive_saw, naive_square, naive_tri, blep_saw, blep_square, wavetable, unison }
    }

    fn process(&mut self, mode: usize) -> f32 {
        match mode {
            0 => self.sine.process(),
            1 => self.naive_saw.process(),
            2 => self.naive_square.process(),
            3 => self.naive_tri.process(),
            4 => self.blep_saw.process(),
            5 => self.blep_square.process(),
            6 => self.wavetable.process(),
            7 => self.unison.process(),
            _ => 0.0,
        }
    }
}

const MODES: &[&str] = &[
    "1  Sine                — pure, computed with sin() each sample",
    "2  Naive Saw           — harsh aliasing at high frequencies",
    "3  Naive Square        — harsh aliasing at high frequencies",
    "4  Naive Triangle      — smooth, no sharp edges, less aliasing",
    "5  PolyBLEP Saw        — anti-aliased version of Naive Saw",
    "6  PolyBLEP Square     — anti-aliased version of Naive Square",
    "7  Wavetable Sine      — lookup table, compare with mode 1",
    "8  Unison x5 +15 cents — five detuned voices, chorus effect",
];

fn print_menu(active: usize) {
    println!("\n=== Chunk 1 — Oscillator Overview (440 Hz) ===");
    for (i, label) in MODES.iter().enumerate() {
        let marker = if i == active { "▶" } else { " " };
        println!("  {} {}", marker, label);
    }
    println!("\n  Type 1–8 and press Enter to switch, q to quit.");
    print!("> ");
}

fn main() -> anyhow::Result<()> {
    let host   = cpal::default_host();
    let device = host.default_output_device().expect("no output device");
    let config = device.default_output_config()?;

    let sample_rate = config.sample_rate() as f32;
    let channels    = config.channels() as usize;

    let mode = Arc::new(Mutex::new(0usize));
    let mode_cb = Arc::clone(&mode);

    let mut oscs = Oscillators::new(sample_rate);

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let m = *mode_cb.lock().unwrap();
            for frame in data.chunks_mut(channels) {
                let sample = oscs.process(m);
                for ch in frame.iter_mut() {
                    *ch = sample;
                }
            }
        },
        |err| eprintln!("audio error: {}", err),
        None,
    )?;

    stream.play()?;

    let mut active = 0;
    print_menu(active);

    loop {
        use std::io::{self, BufRead, Write};
        io::stdout().flush().ok();

        let stdin = io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        let trimmed = line.trim();

        if trimmed == "q" || trimmed == "Q" {
            break;
        }

        if let Ok(n) = trimmed.parse::<usize>() {
            if n >= 1 && n <= MODES.len() {
                active = n - 1;
                *mode.lock().unwrap() = active;
                print_menu(active);
            } else {
                println!("  Enter a number between 1 and {}.", MODES.len());
                print!("> ");
            }
        } else if !trimmed.is_empty() {
            println!("  Unknown input. Type 1–{} or q.", MODES.len());
            print!("> ");
        }
    }

    Ok(())
}
