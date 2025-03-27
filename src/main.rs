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
use std::fs;
use log::*;


fn main() -> Result<()> { 
    // TODO stderrlog can get verbosity from args, either do that or from config or whatever
    stderrlog::new().module(module_path!()).verbosity(log::Level::Info).init().unwrap();

    // Load configuration
    let config_path = Path::new("config.yaml");
    debug!("Loading configuration from: {}", config_path.display());
    let config = Config::load(config_path)?;
    let file_path = "recording";

    cleanup_wav_files(file_path)?;
    
    // Load API key
    let api_key_path = Path::new("apikey.yaml");
    debug!("Loading API key from: {}", api_key_path.display());
    let api_key = ApiKeyConfig::load(api_key_path)?;
    
    debug!("API URL: {}", config.api.url);
    if api_key.key == "YOUR_API_KEY_HERE" {
        warn!("Warning: Using placeholder API key. Please edit apikey.yaml with your actual key, unless you are using a local model.");
    }
    
    info!("Press Ctrl+Shift+Space to start/stop recording");
    
    // Initialize hotkey listener
    let mut hotkey_listener = HotkeyListener::new()?;
    hotkey_listener.setup_hotkey(&config.api.hotkey)?;
    
    // Get shared recording state
    let recording = hotkey_listener.get_recording_state();
    
    // Create shared configuration and API key
    let app_config = Arc::new((config, api_key));
    
    // Start recording management in a separate thread
    let recording_clone = recording.clone();
    let app_config_clone = app_config.clone();
    let recording_thread = thread::spawn(move || -> Result<()> {
        
        debug!("Starting audio thread with recording flag initially: {}", 
                 recording_clone.load(std::sync::atomic::Ordering::SeqCst));
        
        // Start the audio recording system
        // It will internally monitor the recording state flag
        audio::record_audio(file_path, recording_clone, app_config_clone)
    });
    
    debug!("Starting hotkey listener...");
    hotkey_listener.run()?;
    
    // Wait for recording thread to finish TODO this is never reached because we block the main thread
    if let Err(e) = recording_thread.join().unwrap() {
        error!("Recording error: {:?}", e);
    }
    
    Ok(())
}

// Deletes wav files with the given prefix in the current directory
// TODO seems dangerous? maybe pick a prefix that is unlikely to match other files
fn cleanup_wav_files(file_prefix: &str) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let entries = fs::read_dir(&current_dir)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        // Check if it's a file
        if path.is_file() {
            // Check if it's a .wav file with the given prefix
            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                if file_name_str.starts_with(file_prefix) && file_name_str.ends_with(".wav") {
                    debug!("Removing file: {}", path.display());
                    fs::remove_file(path)?;
                }
            }
        }
    }

    Ok(())
    
}