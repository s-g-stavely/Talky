package main

import (
	"fmt"
	"os"
	"path/filepath"

	"gopkg.in/yaml.v2"
)

// Config holds the application configuration
type Config struct {
	Hotkey               string       `yaml:"hotkey"`
	RecordingDurationSec float64      `yaml:"recording_duration_sec"`
	OpenAI               OpenAIConfig `yaml:"openai"`
	Audio                AudioConfig  `yaml:"audio"`
}

// OpenAIConfig holds OpenAI API-related configuration
type OpenAIConfig struct {
	APIKey string `yaml:"api_key,omitempty"`
	Model  string `yaml:"model"`
}

// AudioConfig holds audio recording settings
type AudioConfig struct {
	SampleRate int `yaml:"sample_rate"`
	Channels   int `yaml:"channels"`
	BitDepth   int `yaml:"bit_depth"` // Changed from AudioFormat
}

// LoadConfig loads the application configuration from yaml files
func LoadConfig(configPath string) (*Config, error) {
	// Load main configuration file
	config := &Config{}
	configData, err := os.ReadFile(configPath)
	if err != nil {
		return nil, fmt.Errorf("error reading config file: %w", err)
	}

	err = yaml.Unmarshal(configData, config)
	if err != nil {
		return nil, fmt.Errorf("error parsing config file: %w", err)
	}

	// Load API key from separate file
	apiKeyPath := filepath.Join(filepath.Dir(configPath), "apikey.yaml")
	apiKeyData, err := os.ReadFile(apiKeyPath)
	if err != nil {
		// Only warn about missing API key file, don't fail
		fmt.Printf("Warning: API key file not found at %s\n", apiKeyPath)
	} else {
		var apiKeyConfig struct {
			APIKey string `yaml:"api_key"`
		}
		err = yaml.Unmarshal(apiKeyData, &apiKeyConfig)
		if err != nil {
			return nil, fmt.Errorf("error parsing API key file: %w", err)
		}
		config.OpenAI.APIKey = apiKeyConfig.APIKey
	}

	return config, nil
}
