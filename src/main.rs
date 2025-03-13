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
use config::{Config, ApiKeyConfig};

fn main() -> Result<()> {    
    // Load configuration
    let config_path = Path::new("config.yaml");
    println!("Loading configuration from: {}", config_path.display());
    let config = Config::load(config_path)?;
    
    // Load API key
    let api_key_path = Path::new("apikey.yaml");
    println!("Loading API key from: {}", api_key_path.display());
    let api_key = ApiKeyConfig::load(api_key_path)?;
    
    println!("API URL: {}", config.api.url);
    if api_key.key == "YOUR_API_KEY_HERE" {
        println!("Warning: Using placeholder API key. Please edit apikey.yaml with your actual key.");
    } else {
        println!("API Key: {}...", api_key.key.chars().take(5).collect::<String>());
    }
    
    println!("Press Ctrl+Shift+Space to start/stop recording");
    
    // Initialize hotkey listener
    let mut hotkey_listener = HotkeyListener::new()?;
    hotkey_listener.setup_hotkey()?;
    
    // Get shared recording state
    let recording = hotkey_listener.get_recording_state();
    
    // Create shared configuration and API key
    let app_config = Arc::new((config, api_key));
    
    // Start recording management in a separate thread
    let recording_clone = recording.clone();
    let app_config_clone = app_config.clone();
    let recording_thread = thread::spawn(move || -> Result<()> {
        let file_path = ".";
        
        println!("Starting audio thread with recording flag initially: {}", 
                 recording_clone.load(std::sync::atomic::Ordering::SeqCst));
        
        // Start the audio recording system
        // It will internally monitor the recording state flag
        audio::record_audio(file_path, recording_clone, app_config_clone)
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
