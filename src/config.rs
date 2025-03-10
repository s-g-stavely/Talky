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
    
    /// API temperature parameter
    pub temperature: f32,
    
    /// API temperature increment parameter
    pub temperature_inc: f32,
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
