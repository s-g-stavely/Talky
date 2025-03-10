mod audio;
mod hotkeys;
mod speech;

use std::sync::Arc;
use std::thread;
use anyhow::Result;
use hotkeys::HotkeyListener;

fn main() -> Result<()> {
    println!("Audio recording application");
    println!("Press Ctrl+Shift+Space to start/stop recording");

    // Initialize hotkey listener
    let mut hotkey_listener = HotkeyListener::new()?;
    hotkey_listener.setup_hotkey()?;
    
    // Get shared recording state
    let recording = hotkey_listener.get_recording_state();
    
    // Start recording management in a separate thread
    let recording_clone = recording.clone();
    let recording_thread = thread::spawn(move || -> Result<()> {
        let file_path = "recording";  // Base filename without extension
        
        println!("Starting audio thread with recording flag initially: {}", 
                 recording_clone.load(std::sync::atomic::Ordering::SeqCst));
        
        // Start the audio recording system
        // It will internally monitor the recording state flag
        audio::record_audio(file_path, recording_clone)
    });
    
    println!("Starting hotkey listener...");
    
    // Run the hotkey listener (this will block the main thread)
    hotkey_listener.run()?;
    
    // Wait for recording thread to finish
    if let Err(e) = recording_thread.join().unwrap() {
        eprintln!("Recording error: {:?}", e);
    }
    
    Ok(())
}
