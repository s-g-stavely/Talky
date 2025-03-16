use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use std::io::{BufWriter, Cursor, Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use crate::speech;
use crate::clipboard;
use crate::config::{Config, ApiKeyConfig};

pub fn record_audio(recording_flag: Arc<AtomicBool>, app_config: Arc<(Config, ApiKeyConfig)>) -> Result<()> {
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
    let mut file_opt: Option<Cursor<Vec<u8>>> = None;
    let mut writer_opt: Option<Arc<std::sync::Mutex<hound::WavWriter<Cursor<Vec<u8>>>>>> = None;
    let mut stream_opt: Option<cpal::Stream> = None;
    
    println!("Waiting for hotkey to start recording...");
    println!("Current recording flag state: {}", recording_flag.load(Ordering::SeqCst));
    
    loop { 
        let should_record = recording_flag.load(Ordering::SeqCst);
        
        // Start recording if flag is true and we're not already recording
        if should_record && !stream_active {
            println!("Starting recording");

            let file = Cursor::new(Vec::new());

            // Create in-memory buffer for WAV data
            
            // Create WAV writer with in-memory buffer
            let spec = hound::WavSpec {
                channels: input_config.channels(),
                sample_rate: input_config.sample_rate().0,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            
            let writer = Arc::new(std::sync::Mutex::new(
                hound::WavWriter::new(file, spec)
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

            while recording_flag.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(100));
            }

            
            // Stop and drop the stream first
            if let Some(stream) = stream_opt.take() {
                println!("Stopping audio stream");
                // Explicitly stop the stream before dropping
                if let Err(e) = stream.pause() {
                    eprintln!("Error stopping stream: {:?}", e);
                }
                drop(stream);
            }
            
        } 
        // Stop recording if flag is false and we are currently recording
        else if !should_record && stream_active {
            println!("Flag detected as OFF - stopping recording");
            

            // Process the WAV file
            if let Some(writer_arc) = writer_opt.take() {
                println!("Finalizing WAV file");
                
                // Safely extract the writer
                match Arc::try_unwrap(writer_arc) {
                    Ok(mutex) => {
                        match mutex.into_inner() {
                            Ok(mut writer) => {
                                // Finalize writer to complete WAV header
                                match writer.finalize() {
                                    Ok(()) => {
                                        
                                        // TODO
                                        // // Check if we recorded any data
                                        // if buffer_data.len() > 44 { // 44 bytes is the WAV header size
                                            
                                        // Transcribe in a separate thread
                                        let app_config_clone = app_config.clone();

                                        let mut file_to_transcribe = file_opt.take().unwrap();
                                        
                                        thread::spawn(move || {
                                            match speech::transcribe_audio(&mut file_to_transcribe, &app_config_clone) {
                                                Ok(text) => {
                                                    if !text.is_empty() {
                                                        println!("Transcription: {}", text);
                                                        
                                                        // Copy text to clipboard and paste
                                                        if let Err(e) = clipboard::paste_text(&text) {
                                                            eprintln!("Failed to paste text: {:?}", e);
                                                        } else {
                                                            println!("Successfully pasted transcription");
                                                        }
                                                    } else {
                                                        println!("Transcription returned empty text");
                                                    }
                                                },
                                                Err(e) => eprintln!("Failed to transcribe audio: {:?}", e),
                                            }
                                        });
                                    },
                                    Err(e) => eprintln!("Error finalizing WAV writer: {:?}", e),
                                }
                            },
                            Err(e) => eprintln!("Error accessing WAV writer: {:?}", e),
                        }
                    },
                    Err(_) => eprintln!("Error accessing WAV writer: still in use by other threads"),
                }
            }

            stream_active = false;
            println!("Recording stopped");
        }
        
        // Check every 100ms to avoid busy-waiting
        thread::sleep(Duration::from_millis(100));
    }
}

fn write_input_data<T, U>(input: &[T], writer: &Arc<std::sync::Mutex<hound::WavWriter<Cursor<Vec<u8>>>>>)
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
