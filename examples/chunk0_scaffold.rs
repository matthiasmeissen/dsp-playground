use cpal::traits::*;


fn main() -> anyhow::Result<()> {
    // Get the default audio host for target compilation platform
    let host = cpal::default_host();

    // Try to get the default output device and store success in variable
    let device = host.default_output_device().expect("No output device found");

    println!("Output Device: {}", device.description()?);
    // We use the ? on methods that return a result to avoid writing lines like this
    // match device.description() {
    //     Ok(name) => println!("Output Device: {}", name),
    //     Err(e) => return Err(e.into())
    // }

    // Try get default output config
    let config = device.default_output_config()?;
    println!("Sample Rate: {}", config.sample_rate());
    println!("Channels: {}", config.channels());
    println!("Sample Format: {}", config.sample_format());

    // Build output stream
    let stream = device.build_output_stream(
        &config.into(), 
        // Callback receives mutable buffer slice and fills it with samples
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Write 0.0 to every sample to produce silence
            for sample in data.iter_mut() {
                *sample = 0.0;
            }
        }, 
        move |err| eprintln!("Stream error: {}", err), 
        None
    )?;

    // Start the stream and keep it running by waiting for test input
    stream.play()?;
    println!("Streaming silence. Press Enter to stop.");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // The main function returns a Result
    // Usually there are different result types and we need to handle them individually
    // With using anyhow, this is done automatically
    // By using the ? we make this even simpler
    // When reaching this line we return Ok with nothing meaningful in it
    // Which means we made it without errors
    Ok(())
}
