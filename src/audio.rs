use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::path::Path;
use crate::speech;
use crate::clipboard;
use crate::config::{Config, ApiKeyConfig};

pub fn record_audio(base_path: &str, recording_flag: Arc<AtomicBool>, app_config: Arc<(Config, ApiKeyConfig)>) -> Result<()> {
    // Get the default host
    let host = cpal::default_host();
    
    // Get the default input device
    let device = host.default_input_device()
        .context("Failed to get default input device")?;
    
    println!("Using input device: {}", device.name()?);
    
    // Get the default input config
    let input_config = device.default_input_config()
        .context("Failed to get default input config")?;
    
    println!("Default input config: {:?}", input_config);
    
    let mut stream_active = false;
    let mut writer_opt: Option<Arc<std::sync::Mutex<hound::WavWriter<std::io::BufWriter<File>>>>> = None;
    let mut stream_opt: Option<cpal::Stream> = None;
    let mut current_file_path: Option<String> = None;
    
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
            println!("Starting recording");
            
            let file_path = if base_ext.is_empty() {
                format!("{}.wav", base_stem)
            } else {
                format!("{}.{}", base_stem, base_ext)
            };
            
            println!("Recording to file: {}", file_path);
            current_file_path = Some(file_path.clone());
            
            // Create WAV writer with timestamp in filename
            let spec = hound::WavSpec {
                channels: input_config.channels(),
                sample_rate: input_config.sample_rate().0,
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
            let stream = match input_config.sample_format() {
                SampleFormat::F32 => {
                    let writer = writer.clone();
                    device.build_input_stream(
                        &input_config.config(),
                        move |data: &[f32], _: &_| write_input_data::<f32, i16>(data, &writer),
                        err_fn,
                        None
                    )
                },
                SampleFormat::I16 => {
                    let writer = writer.clone();
                    device.build_input_stream(
                        &input_config.config(),
                        move |data: &[i16], _: &_| write_input_data::<i16, i16>(data, &writer),
                        err_fn,
                        None
                    )
                },
                SampleFormat::U16 => {
                    let writer = writer.clone();
                    device.build_input_stream(
                        &input_config.config(),
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
            
            // Stop and drop the stream first
            if let Some(stream) = stream_opt.take() {
                println!("Stopping audio stream");
                // Explicitly stop the stream before dropping
                if let Err(e) = stream.pause() {
                    eprintln!("Error stopping stream: {:?}", e);
                }
                drop(stream);
            }
            
            // Finalize the WAV file with proper handling
            let file_path_for_transcription = if let Some(writer) = writer_opt.take() {
                println!("Finalizing WAV file");
                
                // Ensure proper finalization
                let file_path = current_file_path.clone();
                let finalized = match writer.lock() {
                    Ok(mut writer_guard) => {
                        // Call flush instead of finalize as it doesn't take ownership
                        if let Err(e) = writer_guard.flush() {
                            eprintln!("Error flushing WAV file: {:?}", e);
                            false
                        } else {
                            println!("WAV file finalized successfully");
                            true
                        }
                        // The writer will be finalized when dropped
                    },
                    Err(e) => {
                        eprintln!("Error locking WAV writer: {:?}", e);
                        false
                    }
                };
                
                if finalized {
                    file_path
                } else {
                    None
                }
            } else {
                None
            };
            
            // Send the file for transcription
            if let Some(file_path) = file_path_for_transcription {
                // Longer delay to ensure file is fully written and closed
                thread::sleep(Duration::from_secs(1));
                
                // Verify the file exists and has content before transcribing
                if let Ok(metadata) = std::fs::metadata(&file_path) {
                    if metadata.len() > 0 {
                        println!("Audio file saved: {} ({} bytes)", file_path, metadata.len());
                        
                        // Transcribe in a separate thread
                        let file_path_clone = file_path.clone();
                        let app_config_clone = app_config.clone();
                        thread::spawn(move || {
                            match speech::transcribe_audio(&file_path_clone, &app_config_clone) {
                                Ok(text) => {
                                    println!("Transcription: {}", text);
                                    
                                    // Copy text to clipboard
                                    if let Err(e) = clipboard::paste_text(&text) {
                                        eprintln!("Failed to paste text: {:?}", e);
                                    } 
                                },
                                Err(e) => eprintln!("Failed to transcribe audio: {}", e),
                            }
                        });
                    } else {
                        eprintln!("Recorded file is empty, skipping transcription");
                    }
                } else {
                    eprintln!("Could not access recorded file, skipping transcription");
                }
            }
            
            stream_active = false;
            current_file_path = None;
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
            if let Err(e) = guard.write_sample(sample) {
                eprintln!("Error writing audio sample: {:?}", e);
                return;
            }
        }
    }
}
