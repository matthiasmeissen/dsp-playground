/// Audio signal diagnostics — drop into an audio callback to monitor
/// peak level, RMS, and DC offset once per second.
///
/// Prints warnings for NaN, infinity, and transients above 1.0.
/// Remove from the signal chain when done debugging.
///
/// # Usage
/// ```
/// use dsp_lib::core::diagnostics::AudioDiagnostics;
///
/// let mut diag = AudioDiagnostics::new(44100.0);
///
/// // In your audio callback:
/// let sample = 0.5_f32;
/// diag.process(sample);
/// ```
pub struct AudioDiagnostics {
    peak: f32,
    sum_squares: f32,
    dc_sum: f32,
    count: usize,
    report_interval: usize,
}

impl AudioDiagnostics {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            peak: 0.0,
            sum_squares: 0.0,
            dc_sum: 0.0,
            count: 0,
            report_interval: sample_rate as usize,
        }
    }

    /// Feed a sample. Prints a report once per second.
    pub fn process(&mut self, sample: f32) {
        // Immediate warnings
        if sample.is_nan() {
            println!("[DIAG] WARNING: NaN detected!");
            return;
        }
        if sample.is_infinite() {
            println!("[DIAG] WARNING: Infinite value detected!");
            return;
        }

        // Accumulate stats
        let abs = sample.abs();
        if abs > self.peak {
            self.peak = abs;
        }
        self.sum_squares += sample * sample;
        self.dc_sum += sample;
        self.count += 1;

        // Report once per second
        if self.count >= self.report_interval {
            let rms = (self.sum_squares / self.count as f32).sqrt();
            let dc_offset = self.dc_sum / self.count as f32;

            let mut warnings = Vec::new();
            if self.peak > 1.0 {
                warnings.push(format!("CLIP peak={:.2}", self.peak));
            }
            if dc_offset.abs() > 0.01 {
                warnings.push(format!("DC={:.4}", dc_offset));
            }

            let warn_str = if warnings.is_empty() {
                String::new()
            } else {
                format!("  !! {}", warnings.join(", "))
            };

            println!(
                "[DIAG] peak={:.3}  rms={:.3}  dc={:.4}{}",
                self.peak, rms, dc_offset, warn_str
            );

            // Reset for next second
            self.peak = 0.0;
            self.sum_squares = 0.0;
            self.dc_sum = 0.0;
            self.count = 0;
        }
    }
}
