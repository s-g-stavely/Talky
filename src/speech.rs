use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::blocking::multipart::{Form, Part};
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use std::io::{Read, Cursor, Seek, SeekFrom};
use std::sync::Arc;
use crate::config::{Config, ApiKeyConfig};

/// Takes an in-memory audio file, sends it to the speech-to-text API,
/// and returns the transcribed text
pub fn transcribe_audio<R: Read + Seek>(audio_data: &mut R, app_config: &Arc<(Config, ApiKeyConfig)>) -> Result<String> {
    let (config, api_key) = &**app_config;
    
    println!("Using API URL: {}", config.api.url);
    
    // Create a reqwest client
    let client = Client::new();
    
    // Read file content into buffer
    let mut file_content = Vec::new();
    audio_data.seek(SeekFrom::Start(0))?;
    audio_data.read_to_end(&mut file_content)
        .context("Failed to read audio data")?;
    
    if file_content.len() <= 44 { // 44 bytes is the WAV header size
        return Err(anyhow::anyhow!("Audio data too small, contains only header"));
    }
    
    println!("Sending audio data ({} bytes) to speech-to-text API", file_content.len());
    
    // Create multipart form
    let mut form = Form::new()
        .part("file", Part::bytes(file_content).file_name("recording.wav"))
        .text("temperature", config.api.temperature.to_string())
        .text("temperature_inc", config.api.temperature_inc.to_string())
        .text("response_format", "json".to_string())
        .text("model", "whisper-1".to_string());

    if !config.api.prompt.is_empty() {
        form = form.text("prompt", config.api.prompt.clone());
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
                // Strip whitespace and return
                Ok(text.trim().to_string())
            } else {
                // Try to extract error message
                if let Some(error) = json.get("error") {
                    Err(anyhow::anyhow!("API error: {:?}", error))
                } else {
                    Err(anyhow::anyhow!("Failed to extract text from response: {}", response_text))
                }
            }
        },
        Err(e) => {
            Err(anyhow::anyhow!("Failed to parse JSON response: {}, body: {}", e, response_text))
        }
    }
}
