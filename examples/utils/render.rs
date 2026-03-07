use hound;

/// Renders a generator as WAV and saves it
pub fn render_to_wav<F>(path: &str, sample_rate: f32, duration_sec: f32, mut generator: F, open_file: bool)
where F: FnMut() -> f32 {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: sample_rate as u32,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(path, spec).expect("Could not create WAV file.");

    let num_samples = (sample_rate * duration_sec) as usize;

    for _ in 0..num_samples {
        writer.write_sample(generator()).expect("Failed to write sample.");
    }

    writer.finalize().expect("Failed to finalize.");
    println!("Wrote {}", path);

    if open_file {
        open_file_in_audacity(path);
    }
}

fn open_file_in_audacity(path: &str) {
    std::process::Command::new("open")
        .arg("-a")
        .arg("Audacity")
        .arg(path)
        .spawn()
        .ok();
}