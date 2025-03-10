mod audio;
mod hotkeys;
mod speech;
mod clipboard;
mod config;

use std::sync::Arc;
use std::thread;
use std::path::Path;
use anyhow::Result;
use hotkeys::HotkeyListener;
use config::Config;

fn main() -> Result<()> {    
    // Load configuration
    let config_path = Path::new("config.yaml");
    println!("Loading configuration from: {}", config_path.display());
    let config = Config::load(config_path)?;
    
    println!("API URL: {}", config.api.url);
    println!("Press Ctrl+Shift+Space to start/stop recording");
    
    // Initialize hotkey listener
    let mut hotkey_listener = HotkeyListener::new()?;
    hotkey_listener.setup_hotkey()?;
    
    // Get shared recording state
    let recording = hotkey_listener.get_recording_state();
    
    // Start recording management in a separate thread
    let recording_clone = recording.clone();
    let config_clone = Arc::new(config.clone());
    let recording_thread = thread::spawn(move || -> Result<()> {
        let file_path = ".";
        
        println!("Starting audio thread with recording flag initially: {}", 
                 recording_clone.load(std::sync::atomic::Ordering::SeqCst));
        
        // Start the audio recording system
        // It will internally monitor the recording state flag
        audio::record_audio(file_path, recording_clone, config_clone)
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
