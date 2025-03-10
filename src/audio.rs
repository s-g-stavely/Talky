use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn record_audio(path: &str, running: Arc<AtomicBool>) -> Result<()> {
    // Get the default host
    let host = cpal::default_host();
    
    // Get the default input device
    let device = host.default_input_device()
        .context("Failed to get default input device")?;
    
    println!("Using input device: {}", device.name()?);
    
    // Get the default input config
    let config = device.default_input_config()
        .context("Failed to get default input config")?;
    
    println!("Default input config: {:?}", config);
    
    // Create a WAV writer
    let spec = hound::WavSpec {
        channels: config.channels(),
        sample_rate: config.sample_rate().0,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let writer = Arc::new(std::sync::Mutex::new(
        hound::WavWriter::create(path, spec)
            .context("Failed to create WAV writer")?
    ));
    
    // Create a callback for the input stream
    let err_fn = |err| eprintln!("an error occurred on the input audio stream: {}", err);
    
    // Build the input stream
    let stream = match config.sample_format() {
        SampleFormat::F32 => {
            let writer = Arc::clone(&writer);
            device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &_| write_input_data::<f32, i16>(data, &writer),
                err_fn,
                None
            )
        },
        SampleFormat::I16 => {
            let writer = Arc::clone(&writer);
            device.build_input_stream(
                &config.into(),
                move |data: &[i16], _: &_| write_input_data::<i16, i16>(data, &writer),
                err_fn,
                None
            )
        },
        SampleFormat::U16 => {
            let writer = Arc::clone(&writer);
            device.build_input_stream(
                &config.into(),
                move |data: &[u16], _: &_| write_input_data::<u16, i16>(data, &writer),
                err_fn,
                None
            )
        },
        _ => return Err(anyhow::anyhow!("Unsupported sample format")),
    }.context("Failed to build input stream")?;
    
    // Start the stream
    stream.play()?;
    
    // Keep the stream alive while recording
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }
    
    // The stream will be dropped when it goes out of scope, stopping the recording
    drop(stream);
    
    // Lock and drop the writer to ensure it's properly finalized
    let writer_guard = writer.lock().unwrap();
    drop(writer_guard);
    
    Ok(())
}

fn write_input_data<T, U>(input: &[T], writer: &Arc<std::sync::Mutex<hound::WavWriter<std::io::BufWriter<File>>>>)
where
    T: Sample,
    U: Sample + hound::Sample + cpal::FromSample<T>,
{
    if let Ok(mut guard) = writer.try_lock() {
        for &sample in input.iter() {
            let sample: U = sample.to_sample();
            guard.write_sample(sample).unwrap();
        }
    }
}
