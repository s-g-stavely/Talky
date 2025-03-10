mod audio;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{io, thread};
use anyhow::Result;

fn main() -> Result<()> {
    println!("Audio recording application");
    println!("Press Enter to start recording");
    println!("Press Ctrl+C to stop recording");

    // Wait for user to press Enter
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;

    // Create a flag to control recording
    let recording = Arc::new(AtomicBool::new(true));
    let recording_clone = recording.clone();

    // Setup Ctrl+C handler
    ctrlc::set_handler(move || {
        recording_clone.store(false, Ordering::SeqCst);
        println!("Stopping recording...");
    })?;

    // Start recording
    let file_path = "recorded_audio.wav";
    println!("Recording started. Saving to {}", file_path);
    audio::record_audio(file_path, recording)?;

    println!("Recording stopped and saved to {}", file_path);
    Ok(())
}
