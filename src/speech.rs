use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::blocking::multipart::{Form, Part};
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::thread;
use std::time::Duration;

/// Takes a path to an audio file, sends it to the speech-to-text API,
/// and returns the transcribed text
pub fn transcribe_audio(file_path: &str) -> Result<String> {
    println!("Preparing to transcribe audio file: {}", file_path);
    
    // Wait a bit longer to ensure file is completely written and closed
    thread::sleep(Duration::from_millis(1000));
    
    // Create a reqwest client
    let client = Client::new();
    
    // Verify the file exists and has content
    let file_path = Path::new(file_path);
    let metadata = std::fs::metadata(file_path)
        .context(format!("Failed to get metadata for file: {}", file_path.display()))?;
    
    if metadata.len() == 0 {
        return Err(anyhow::anyhow!("Audio file is empty"));
    }
    
    println!("Audio file size: {} bytes", metadata.len());
    
    // Open the file
    let mut file = File::open(file_path)
        .context(format!("Failed to open file: {}", file_path.display()))?;
    
    // Read file content
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content)
        .context("Failed to read file content")?;
    
    // Get the filename for the form
    let file_name = file_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("audio.wav");
    
    println!("Sending file '{}' to speech-to-text API", file_name);
    
    // Create multipart form
    let form = Form::new()
        .part("file", Part::bytes(file_content).file_name(file_name.to_string()))
        .text("temperature", "0.0")
        .text("temperature_inc", "0.2")
        .text("response_format", "json");
    
    // Send the request
    let response = client.post("http://127.0.0.1:8080/inference")
        .multipart(form)
        .send()
        .context("Failed to send request to speech-to-text API")?;
    
    // Check if the request was successful
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().unwrap_or_default();
        return Err(anyhow::anyhow!(
            "API request failed with status: {}, body: {}", 
            status,
            error_text
        ));
    }
    
    // Parse the response
    let response_text = response.text().context("Failed to read response text")?;
    println!("Raw API response: {}", response_text);
    
    // Try to parse as JSON to extract the text field
    match serde_json::from_str::<Value>(&response_text) {
        Ok(json) => {
            if let Some(text) = json.get("text").and_then(|t| t.as_str()) {
                Ok(text.to_string())
            } else {
                // If we can't find a "text" field, just return the raw response
                Ok(response_text)
            }
        },
        Err(_) => {
            // If it's not valid JSON, return the raw response
            Ok(response_text)
        }
    }
}
