use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::blocking::multipart::{Form, Part};
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use crate::config::{Config, ApiKeyConfig};

/// Takes a path to an audio file, sends it to the speech-to-text API,
/// and returns the transcribed text
pub fn transcribe_audio(file_path: &str, app_config: &Arc<(Config, ApiKeyConfig)>) -> Result<String> {
    let (config, api_key) = &**app_config;
    
    println!("Preparing to transcribe audio file: {}", file_path);
    println!("Using API URL: {}", config.api.url);
    
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
    let mut form = Form::new()
        .part("file", Part::bytes(file_content).file_name(file_name.to_string()))
        .text("temperature", config.api.temperature.to_string())
        .text("temperature_inc", config.api.temperature_inc.to_string())
        .text("response_format", "json".to_string())
        .text("model", config.api.model.to_string());

    if config.api.prompt != "" {
        form = form.text("prompt", config.api.prompt.to_string());
    }
    
    // Check if the API key is already in Bearer format
    let api_key_value = if api_key.key.starts_with("Bearer ") {
        api_key.key.clone()
    } else {
        format!("Bearer {}", api_key.key)
    };
    
    // Set the API key
    let mut headers = HeaderMap::new();
    if let Ok(header_value) = HeaderValue::from_str(&api_key_value) {
        headers.insert("Authorization", header_value);
    }
    
    // Send the request to configured URL with API key in header
    let response = client.post(&config.api.url)
        .headers(headers)
        .multipart(form)
        .send()
        .context("Failed to send request to speech-to-text API")?;

    // Delete the file
    std::fs::remove_file(file_path)
        .context(format!("Failed to delete file: {}", file_path.display()))?;
    
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

                // Strip whitespace and return. We leave a space at the end so that 
                // there's a space between this transcription and the next one
                Ok(text.trim().to_string() + " ")
            } else {
                // TODO maybe try to extract error message
                Err(anyhow::anyhow!(response_text))
            }
        },
        Err(_) => {
            Err(anyhow::anyhow!(response_text))
        }
    }
}
