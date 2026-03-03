# Chunk 0 — Scaffold

**Day budget:** 4 days
**Status:** ⬜ Not started

### What does it do?
Sets up the audio pipeline: CPAL opens a stream to your audio hardware, your callback function is called repeatedly to fill audio buffers, and sound comes out of your speakers. This is the foundation that every subsequent module plugs into. Getting this right — and understanding *why* it works the way it does — makes everything else feel natural.

### What are the knobs?
Sample rate (typically 44100 or 48000 Hz), buffer size (how many samples per callback), output device selection.

### How does it connect?
This is the root of the entire signal chain. Every other module's `process()` eventually gets called from inside the CPAL callback. Nothing else in the library touches audio hardware — only this layer does.

### The mental model: what actually happens when you play audio

Before writing any code, it helps to understand the flow clearly.

Your CPU and your audio hardware run at different rhythms. The hardware's DAC (digital-to-analog converter) consumes samples at a fixed, relentless rate — say 44100 samples per second. It never pauses. Your job is to keep it fed.

CPAL manages this by maintaining a **buffer**: a small array of pre-computed samples. The hardware reads from one end; your callback writes to the other. When the hardware has consumed enough samples, CPAL calls your callback to refill the buffer. This is called the **audio callback pattern** and it is the foundation of almost all real-time audio programming, in any language, on any platform.

```
Your callback → fills buffer → hardware reads buffer → DAC → speakers
                    ↑                                           |
                    └──────────── CPAL manages this loop ───────┘
```

The callback has hard timing constraints. If it takes too long to run, the hardware runs out of samples and you get a dropout — a click or pop in the audio. This is called a **buffer underrun**. It means: keep your callback lean, no allocations, no blocking, no file I/O.

**Key numbers to internalize:**
- At 44100 Hz, each sample lasts ~22.7 microseconds
- A typical buffer of 512 samples gives you ~11.6 milliseconds to fill it
- That 11.6ms is your entire computational budget per callback

### The mental model: sample rate and frequency

Sample rate determines what frequencies are representable. By the Nyquist theorem, you can represent any frequency up to **half the sample rate**. At 44100 Hz, that's 22050 Hz — comfortably above human hearing (~20 Hz to 20 kHz).

You don't need to deeply understand the math. The practical consequence is: never try to generate a frequency above `sample_rate / 2` or you'll get aliasing artifacts. Your oscillator code will naturally stay in range as long as you're working with musical frequencies.

**Sample rate also determines time.** One second of audio = `sample_rate` samples. Half a second = `sample_rate / 2` samples. This is the conversion you'll use constantly:

```
time_in_samples = time_in_seconds * sample_rate
frequency_per_sample = frequency_in_hz / sample_rate
```

### Reference material
- [CPAL docs](https://docs.rs/cpal)
- [CPAL examples on GitHub](https://github.com/RustAudio/cpal/tree/master/examples) — the `beep` example is your starting point
- [The Audio Programmer on YouTube](https://www.youtube.com/@TheAudioProgrammer) — good conceptual background

---

### Daily steps

#### Day 0.1 — Project setup and core traits

**Goal:** Create the library skeleton and define the shared language that all modules will speak.

**Set up the project:**

```bash
cargo new dsp-lib --lib
cd dsp-lib
mkdir -p src/core src/oscillator src/filter src/envelope src/noise src/delay src/clock src/effects src/granular
mkdir examples
```

**`Cargo.toml`** — notice that `dsp-lib` itself has **no** dependencies. CPAL lives only in the examples:

```toml
[package]
name = "dsp-lib"
version = "0.1.0"
edition = "2021"

# The core library has NO dependencies — pure math, pure f32.
# This keeps it portable to embedded targets.
[dependencies]

# These are only compiled for examples and tests, never the library itself.
[dev-dependencies]
cpal   = "0.15"   # audio I/O for desktop examples
hound  = "3.5"    # write .wav files for visual debugging
anyhow = "1.0"    # error handling convenience in examples
```

> **Why keep CPAL out of the library?** If `dsp-lib` depends on CPAL, it can never run on embedded targets — CPAL requires `std` and an OS audio driver. Keeping the library pure means the exact same `Sine`, `Svf`, and `Envelope` structs will compile on a Raspberry Pi Pico later with zero changes. The hardware glue layer is always a separate concern.

**`src/core/traits.rs`** — define the shared traits:

```rust
/// A module that generates or transforms one audio sample at a time.
/// Generators (oscillators, noise) implement this with no input.
/// Processors (filters, effects) need a separate input — see Transform below.
pub trait Process {
    fn process(&mut self) -> f32;
}

/// A module that transforms one sample at a time.
/// Takes input audio, returns processed audio.
pub trait Transform {
    fn process(&mut self, input: f32) -> f32;
}

/// Anything that can be reset to its initial state (phase, buffer, envelope).
pub trait Reset {
    fn reset(&mut self);
}

/// Anything that needs to know the sample rate to compute its coefficients.
pub trait SetSampleRate {
    fn set_sample_rate(&mut self, sample_rate: f32);
}
```

> **Why two traits for processing?** Generators produce signal from nothing — oscillators, noise sources, envelope followers. Processors take signal in and return signal out — filters, delays, effects. Keeping them separate makes the types honest and the signal chain readable.

**`src/core.rs`:**

```rust
pub mod traits;
pub use traits::*;
```

**`src/lib.rs`:**

```rust
pub mod core;
pub mod oscillator;
// (add more as you build them)

pub use core::traits::*;
```

**`src/oscillator.rs`** — create an empty placeholder:

```rust
// modules added as you build them
```

At the end of Day 0.1 the project should compile cleanly with `cargo build`.

---

#### Day 0.2 — CPAL hello world (silence)

**Goal:** Open an audio stream and confirm the pipeline works before generating any sound.

Create `examples/chunk0_scaffold.rs`:

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};

fn main() -> anyhow::Result<()> {
    // 1. Get the default audio host (CoreAudio on macOS, WASAPI on Windows, ALSA on Linux)
    let host = cpal::default_host();

    // 2. Get the default output device
    let device = host
        .default_output_device()
        .expect("No output device found");

    println!("Output device: {}", device.name()?);

    // 3. Get the default output config — this tells you the sample rate, channels, format
    let config = device.default_output_config()?;
    println!("Sample rate: {}", config.sample_rate().0);
    println!("Channels: {}", config.channels());
    println!("Sample format: {:?}", config.sample_format());

    // 4. Build a stream that outputs silence
    // The callback receives a mutable buffer slice and fills it with samples.
    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Fill every sample with silence
            for sample in data.iter_mut() {
                *sample = 0.0;
            }
        },
        move |err| eprintln!("Stream error: {}", err),
        None,
    )?;

    // 5. Start the stream and keep it running
    stream.play()?;
    println!("Streaming silence. Press Enter to stop.");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}
```

Run it: `cargo run --example chunk0_scaffold`

You should see the device name and config printed, and the program waits for you to press Enter. No sound yet — that's correct. Confirm there are no errors.

> **What's `data: &mut [f32]`?** This is your buffer. CPAL hands you a mutable slice of `f32` values. The length is `buffer_size * channels`. If you have stereo output and a buffer of 512 samples, `data` has 1024 elements — left/right interleaved: `[L, R, L, R, ...]`. You write into it and CPAL sends it to the hardware.

---

#### Day 0.3 — Sine wave in the callback

**Goal:** Produce your first sound and understand why the callback must be stateful.

The key problem: to generate a continuous sine wave across many callback invocations, you need to remember where in the wave cycle you were at the end of the last callback. This is **state**, and it needs to live somewhere that persists between calls.

In Rust, the idiomatic way is to capture state in the closure via `move`:

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32::consts::TAU;

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device");
    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;

    // State that lives across callback invocations
    let mut phase: f32 = 0.0;
    let frequency: f32 = 440.0; // A4
    let phase_increment = frequency / sample_rate;

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // data contains interleaved samples for all channels
            // chunk_exact(channels) groups them by frame (one sample per channel)
            for frame in data.chunks_mut(channels) {
                // Compute one sample of a sine wave
                let sample = (phase * TAU).sin() * 0.3; // 0.3 = amplitude, keeps it quiet

                // Advance phase; wrap at 1.0 to avoid float drift
                phase = (phase + phase_increment) % 1.0;

                // Write same sample to all channels (mono signal to stereo output)
                for channel_sample in frame.iter_mut() {
                    *channel_sample = sample;
                }
            }
        },
        move |err| eprintln!("Stream error: {}", err),
        None,
    )?;

    stream.play()?;
    println!("Playing 440Hz sine. Press Enter to stop.");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}
```

Run it. You should hear a clean 440Hz tone (concert A).

**Understand what just happened:**
- `phase` advances by `frequency / sample_rate` each sample — this is the phase accumulator pattern you will use in every oscillator forever
- The `move` closure captures `phase` by value and mutates it across calls
- `% 1.0` keeps phase in the 0..1 range — without this it would drift to large floats and lose precision over time
- `* 0.3` scales amplitude — a full-scale sine at `1.0` is very loud; `0.3` is a reasonable level for testing

**Exercise:** Change `frequency` to 220.0 (an octave lower) and 880.0 (an octave higher). Notice the octave relationship — doubling frequency = one octave up.

---

#### Day 0.4 — Move state into a struct

**Goal:** Extract the phase accumulator into your first real DSP module, and confirm that every future module will follow the same pattern.

This is the most important structural day. The inline closure approach from Day 0.3 doesn't scale — you can't easily compose multiple modules that way. The solution is to move all state into a struct and implement your `Process` trait on it.

**`src/oscillator/sine.rs`:**

```rust
use crate::core::traits::{Process, Reset, SetSampleRate};
use std::f32::consts::TAU;

pub struct Sine {
    phase: f32,
    phase_increment: f32,
    amplitude: f32,
    sample_rate: f32,
}

impl Sine {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            phase: 0.0,
            phase_increment: 0.0,
            amplitude: 1.0,
            sample_rate,
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.phase_increment = freq / self.sample_rate;
    }

    pub fn set_amplitude(&mut self, amp: f32) {
        self.amplitude = amp;
    }
}

impl Process for Sine {
    fn process(&mut self) -> f32 {
        let out = (self.phase * TAU).sin() * self.amplitude;
        self.phase = (self.phase + self.phase_increment) % 1.0;
        out
    }
}

impl Reset for Sine {
    fn reset(&mut self) {
        self.phase = 0.0;
    }
}

impl SetSampleRate for Sine {
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        // Recalculate increment if frequency was already set
        // For now, just store — caller should re-set frequency after changing sample rate
    }
}
```

**Update `src/oscillator.rs`** — declare the new submodule:

```rust
pub mod sine;
pub use sine::Sine;
```

**Update the example** to use the struct:

```rust
use dsp_lib::oscillator::Sine;
use dsp_lib::core::traits::Process;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device");
    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;

    // Create and configure the oscillator
    let mut osc = Sine::new(sample_rate);
    osc.set_frequency(440.0);
    osc.set_amplitude(0.3);

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let sample = osc.process(); // clean, simple, composable
                for ch in frame.iter_mut() {
                    *ch = sample;
                }
            }
        },
        move |err| eprintln!("Stream error: {}", err),
        None,
    )?;

    stream.play()?;
    println!("Playing 440Hz via Sine struct. Press Enter to stop.");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}
```

This is the template for every module going forward. Notice how clean the callback has become — it just calls `.process()`. Adding a filter in Chunk 2 will be as simple as:

```rust
let filtered = filter.process(osc.process());
```

**Exercise:** Create a second `Sine` at 660.0 Hz, call `.process()` on both in the callback, and add their outputs together (multiply the sum by 0.5 to keep the amplitude in range). You now have a two-oscillator synthesizer.

---

#### Day 0.5 — Visualization helper (optional but strongly recommended)

**Goal:** Build a `render_to_wav` utility so you can debug DSP math visually — by opening a file in Audacity — rather than purely by ear.

Listening is fine for "does this make a sound?" but not for "is my envelope shape correct?" or "why is my filter ringing unexpectedly?". A `.wav` renderer closes that gap and is the fastest debugging tool you have for the chunks ahead.

Add `hound = "3.5"` to your `[dev-dependencies]` if not already there. Then create `examples/utils/render.rs` (or just a free function in your example files):

```rust
use hound::{WavWriter, WavSpec, SampleFormat};

/// Render `duration_secs` seconds of audio by repeatedly calling `generator`.
/// Writes a mono 44100Hz 32-bit float WAV to `path`.
pub fn render_to_wav<F>(path: &str, sample_rate: f32, duration_secs: f32, mut generator: F)
where
    F: FnMut() -> f32,
{
    let spec = WavSpec {
        channels: 1,
        sample_rate: sample_rate as u32,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };
    let mut writer = WavWriter::create(path, spec).expect("Could not create WAV file");
    let num_samples = (sample_rate * duration_secs) as usize;
    for _ in 0..num_samples {
        writer.write_sample(generator()).expect("Write failed");
    }
    writer.finalize().expect("Finalize failed");
    println!("Wrote {}", path);
}
```

Use it like this to render 2 seconds of a 440Hz sine to a file:

```rust
let mut osc = Sine::new(44100.0);
osc.set_frequency(440.0);
osc.set_amplitude(0.5);

render_to_wav("output/sine_440.wav", 44100.0, 2.0, || osc.process());
```

Open the file in Audacity (`File → Import → Audio`) and switch to the spectrogram view to see frequency content. You will use this constantly from Chunk 2 onwards to verify filter slopes, envelope shapes, and aliasing.

> **Gain staging reminder:** Any sample value outside -1.0..1.0 will clip when written to WAV and will sound harshly distorted. If you add multiple oscillators together, divide the sum by the number of voices. For a soft safety net, you can wrap the output in a `tanh` — `out.tanh()` — which gently limits to -1..1 while adding a small amount of harmonic saturation. This is not a substitute for correct gain management but it will save you from harsh digital clipping during experimentation.

---

#### Day 0.6 — Thread communication: the Remote Control pattern

**Goal:** Change the frequency of the sine wave *while it is playing* by typing into the console — and understand the pattern that makes this safe.

Right now, `osc` is moved into the CPAL closure and lives there forever. You can set parameters before hitting play, but you cannot change them while audio is running — the struct is inaccessible from the main thread once moved.

For a learning curriculum where you build and audition one thing at a time, this is completely fine. Restart the program to try a new frequency. But once you want to sweep a filter, tune a pitch, or wire anything to a knob, you need a proper solution.

**The problem with `Mutex`.** The obvious fix — wrapping the oscillator in a `Mutex` — is specifically wrong for audio. A `Mutex` blocks when another thread holds the lock. If the main thread holds the lock for even a few milliseconds (say, during a memory allocation), the audio callback stalls, the hardware runs out of samples, and you get a pop or dropout. The audio thread must never wait.

**The solution: atomics.** An atomic operation is a read or write that completes in a single CPU instruction — it cannot be interrupted mid-way. No locks, no waiting. Standard Rust gives us `AtomicU32`; since there is no `AtomicF32` in `std`, we bit-cast the float to `u32` for storage. This is a standard audio programming trick.

**Step 1: Create `src/core/param.rs`**

```rust
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// A thread-safe, lock-free shared parameter.
/// Write from the main/UI thread with `.set()`, read from the audio thread with `.get()`.
/// Clone it freely — all clones point to the same underlying value.
#[derive(Clone)]
pub struct SharedParam {
    inner: Arc<AtomicU32>,
}

impl SharedParam {
    pub fn new(initial_value: f32) -> Self {
        Self {
            inner: Arc::new(AtomicU32::new(initial_value.to_bits())),
        }
    }

    /// Call from the main/UI thread to update the value.
    pub fn set(&self, value: f32) {
        self.inner.store(value.to_bits(), Ordering::Relaxed);
    }

    /// Call from the audio thread to read the current value.
    pub fn get(&self) -> f32 {
        f32::from_bits(self.inner.load(Ordering::Relaxed))
    }
}
```

> **Why `Ordering::Relaxed`?** There are several memory ordering options in Rust atomics. `Relaxed` is sufficient here because each parameter is independent — we only need the float read/write to be atomic, not sequentially consistent with other operations. For a deeper dive, see the Rustonomicon on atomics. For now: `Relaxed` is the right choice for audio parameters.

**Step 2: Add to `src/core.rs`** — declare the new submodule:

```rust
pub mod traits;
pub mod param;
pub use traits::*;
pub use param::SharedParam;
```

**Step 3: Extend `Sine` to hold a `SharedParam`**

Rather than baking `SharedParam` into every struct (which would complicate the embedded use case), keep it as an optional pattern in the examples layer. The cleanest approach is to read from the param in the callback and call `set_frequency` on the oscillator — the struct stays clean, the wiring lives in the runner:

```rust
// In the audio callback — read atomic, update oscillator, then process
move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
    let freq = freq_param_audio.get();          // lock-free read
    osc.set_frequency(freq);                     // update oscillator state
    for frame in data.chunks_mut(channels) {
        let sample = osc.process() * 0.2;
        for ch in frame.iter_mut() { *ch = sample; }
    }
}
```

> **Note on calling `set_frequency` every callback vs every sample:** Calling it once per buffer (not once per sample) means the frequency is fixed for the duration of that buffer. At a buffer size of 512 samples at 44100Hz, that is ~11ms of latency before a frequency change takes effect. This is imperceptible in practice. If you want truly sample-accurate parameter changes you would read the atomic inside the inner sample loop — but that is rarely necessary.

**Step 4: Build the interactive example**

Create `examples/chunk0_interactive.rs`:

```rust
use dsp_lib::core::param::SharedParam;
use dsp_lib::core::traits::Process;
use dsp_lib::oscillator::Sine;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device");
    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;

    // Create the shared parameter — one end stays here, one goes to the audio thread
    let freq_param = SharedParam::new(440.0);
    let freq_param_audio = freq_param.clone(); // Arc clone: same value, different handle

    let mut osc = Sine::new(sample_rate);
    osc.set_frequency(440.0);
    osc.set_amplitude(0.2);

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Read the current frequency atomically — no lock, no wait
            osc.set_frequency(freq_param_audio.get());
            for frame in data.chunks_mut(channels) {
                let sample = osc.process();
                for ch in frame.iter_mut() { *ch = sample; }
            }
        },
        move |err| eprintln!("Stream error: {}", err),
        None,
    )?;

    stream.play()?;
    println!("Playing 440Hz. Type a frequency in Hz and press Enter to change pitch.");
    println!("Try: 220  440  880  261  329  392");
    println!("Type 'q' to quit.");

    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        stdin.read_line(&mut input)?;
        let trimmed = input.trim();
        if trimmed == "q" { break; }
        match trimmed.parse::<f32>() {
            Ok(freq) => {
                println!("→ {:.1} Hz", freq);
                freq_param.set(freq); // atomic write — audio thread picks it up instantly
            }
            Err(_) => println!("Enter a number (Hz) or 'q' to quit"),
        }
    }

    Ok(())
}
```

Run it with `cargo run --example chunk0_interactive` and type frequencies while listening. The pitch changes without any click, dropout, or restarting the program.

**What you just built:**
- `SharedParam` is the foundation of every "knob" in a real-time audio application
- `Arc` handles shared ownership across threads; `AtomicU32` handles safe concurrent access
- The oscillator struct itself stays clean — it knows nothing about threading
- This same `SharedParam` type will wire LFOs to filter cutoffs, envelopes to amplitude, and eventually GUI sliders to any parameter in the library

**When to go further:** For sending discrete events (note on/off, sequence steps, preset changes) rather than continuous parameter values, look at the `rtrb` crate — a real-time safe lock-free ring buffer purpose-built for audio thread communication.

---

### Chunk 0 checklist
- [ ] Project compiles cleanly: `cargo build`
- [ ] Core traits defined: `Process`, `Transform`, `Reset`, `SetSampleRate`
- [ ] `Cargo.toml`: CPAL is in `[dev-dependencies]` only — library has no external dependencies
- [ ] CPAL stream opens and streams silence without errors
- [ ] 440Hz sine tone plays through speakers
- [ ] `Sine` struct in `src/oscillator/sine.rs` implements all four traits
- [ ] Callback only calls `.process()` — no DSP math inline
- [ ] (Optional) `render_to_wav` helper works and produces a valid WAV file you can open in Audacity
- [ ] (Optional) `chunk0_interactive` example runs — typing a frequency changes the pitch without dropout
- [ ] `SharedParam` struct exists in `src/core/param.rs` with `set()` and `get()` methods
- [ ] You can explain: sample rate, Nyquist, the phase accumulator, buffer underrun, why CPAL stays out of the library
- [ ] You can explain: why `Mutex` is wrong in audio callbacks, why atomics are right, what `Arc::clone` gives you
- [ ] You understand: parameter smoothing will be needed from Chunk 2 onward, denormals must be flushed in every feedback structure

---