use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use std::path::Path;

pub fn record_audio(base_path: &str, recording_flag: Arc<AtomicBool>) -> Result<()> {
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
    
    let mut stream_active = false;
    let mut writer_opt: Option<Arc<std::sync::Mutex<hound::WavWriter<std::io::BufWriter<File>>>>> = None;
    let mut stream_opt: Option<cpal::Stream> = None;
    
    println!("Waiting for hotkey to start recording...");
    println!("Current recording flag state: {}", recording_flag.load(Ordering::SeqCst));
    
    // Main processing loop that monitors the recording flag
    let base_path = Path::new(base_path);
    let base_stem = base_path.file_stem().unwrap_or_default().to_string_lossy();
    let base_ext = base_path.extension().unwrap_or_default().to_string_lossy();
    
    loop { 
        let should_record = recording_flag.load(Ordering::SeqCst);
        
        // Start recording if flag is true and we're not already recording
        if should_record && !stream_active {
            println!("Flag detected as ON - starting recording");
            
            // Create a timestamped filename
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            let file_path = if base_ext.is_empty() {
                format!("{}_{}.wav", base_stem, timestamp)
            } else {
                format!("{}_{}.{}", base_stem, timestamp, base_ext)
            };
            
            println!("Starting new recording to file: {}", file_path);
            
            // Create WAV writer with timestamp in filename
            let spec = hound::WavSpec {
                channels: config.channels(),
                sample_rate: config.sample_rate().0,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            
            let writer = Arc::new(std::sync::Mutex::new(
                hound::WavWriter::create(&file_path, spec)
                    .context("Failed to create WAV writer")?
            ));
            
            writer_opt = Some(writer.clone());
            
            // Create error callback
            let err_fn = |err| eprintln!("An error occurred on the input audio stream: {}", err);
            
            // Build the input stream
            let stream = match config.sample_format() {
                SampleFormat::F32 => {
                    let writer = writer.clone();
                    device.build_input_stream(
                        &config.config(),
                        move |data: &[f32], _: &_| write_input_data::<f32, i16>(data, &writer),
                        err_fn,
                        None
                    )
                },
                SampleFormat::I16 => {
                    let writer = writer.clone();
                    device.build_input_stream(
                        &config.config(),
                        move |data: &[i16], _: &_| write_input_data::<i16, i16>(data, &writer),
                        err_fn,
                        None
                    )
                },
                SampleFormat::U16 => {
                    let writer = writer.clone();
                    device.build_input_stream(
                        &config.config(),
                        move |data: &[u16], _: &_| write_input_data::<u16, i16>(data, &writer),
                        err_fn,
                        None
                    )
                },
                _ => return Err(anyhow::anyhow!("Unsupported sample format")),
            }.context("Failed to build input stream")?;
            
            // Start the stream
            stream.play()?;
            stream_opt = Some(stream);
            stream_active = true;
            println!("Audio stream activated successfully");
        } 
        // Stop recording if flag is false and we are currently recording
        else if !should_record && stream_active {
            println!("Flag detected as OFF - stopping recording");
            
            // Stop and drop the stream
            if let Some(stream) = stream_opt.take() {
                println!("Dropping audio stream");
                drop(stream);
            }
            
            // Finalize the WAV file
            if let Some(writer) = writer_opt.take() {
                println!("Finalizing WAV file");
                // Lock and drop the writer to ensure it's properly finalized
                match writer.lock() {
                    Ok(writer_guard) => {
                        drop(writer_guard);
                        println!("WAV file finalized successfully");
                    },
                    Err(e) => {
                        println!("Error locking WAV writer: {:?}", e);
                    }
                }
            }
            
            stream_active = false;
            println!("Recording stopped and saved.");
        }
        
        // Check every 100ms to avoid busy-waiting
        thread::sleep(Duration::from_millis(100));
    }
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
