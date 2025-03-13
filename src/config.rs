use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

/// Application configuration loaded from config.yaml
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// API configuration
    pub api: ApiConfig,
}

/// API-related configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiConfig {
    /// URL for the speech-to-text API
    pub url: String,


    // API parameters
    pub prompt: String,
    pub temperature: f32,
    pub temperature_inc: f32,
}

/// API Key configuration loaded from apikey.yaml
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiKeyConfig {
    /// API key for authentication
    pub key: String,
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            key: "YOUR_API_KEY_HERE".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from the specified path
    pub fn load(path: &Path) -> Result<Self> {
        // Check if the file exists
        if path.exists() {
            // Open and read the file
            let mut file = File::open(path)
                .context(format!("Failed to open config file: {}", path.display()))?;
            
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .context("Failed to read config file")?;
            
            // Parse YAML
            let config: Config = serde_yaml::from_str(&contents)
                .context("Failed to parse config file")?;

            
            Ok(config)
        } else {
            Err(anyhow::anyhow!("Config file not found: {}", path.display()))
        }
    }
}

impl ApiKeyConfig {
    /// Load API key configuration from the specified path
    /// If the file doesn't exist, create a default one
    pub fn load(path: &Path) -> Result<Self> {
        // Check if the file exists
        if path.exists() {
            // Open and read the file
            let mut file = File::open(path)
                .context(format!("Failed to open API key file: {}", path.display()))?;
            
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .context("Failed to read API key file")?;
            
            // Parse YAML
            let mut api_key_config: ApiKeyConfig = serde_yaml::from_str(&contents)
                .context("Failed to parse API key file")?;
            
            Ok(api_key_config)
        } else {
            // Create a default config
            let api_key = ApiKeyConfig::default();
            ApiKeyConfig::save(&api_key, path)?;
            
            println!("Created default API key file at: {}", path.display());
            println!("⚠️  Please edit this file and add your actual API key.");
            
            Ok(api_key)
        }
    }
    
    /// Save API key configuration to the specified path
    pub fn save(&self, path: &Path) -> Result<()> {
        // Serialize to YAML
        let yaml = serde_yaml::to_string(self)
            .context("Failed to serialize API key to YAML")?;
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .context(format!("Failed to create directory: {}", parent.display()))?;
            }
        }
        
        // Write to file
        let mut file = File::create(path)
            .context(format!("Failed to create API key file: {}", path.display()))?;
        
        file.write_all(yaml.as_bytes())
            .context("Failed to write API key file")?;
        
        Ok(())
    }
}
