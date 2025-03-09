package main

import (
	"io/ioutil"

	"gopkg.in/yaml.v2"
)

// Config holds the application configuration
type Config struct {
	// Hotkey combination to trigger recording (e.g., "ctrl+shift+space")
	Hotkey string `yaml:"hotkey"`

	// Recording duration in seconds
	RecordingDurationSec float64 `yaml:"recording_duration_sec"`

	// OpenAI API configuration
	OpenAI struct {
		APIKey string `yaml:"api_key"`
		Model  string `yaml:"model"` // whisper-1 or other variants
	} `yaml:"openai"`

	// Audio settings
	Audio struct {
		SampleRate  int `yaml:"sample_rate"`
		Channels    int `yaml:"channels"`
		AudioFormat int `yaml:"audio_format"`
	} `yaml:"audio"`
}

// LoadConfig reads the configuration file and returns a Config struct
func LoadConfig(filename string) (*Config, error) {
	data, err := ioutil.ReadFile(filename)
	if err != nil {
		return nil, err
	}

	config := &Config{}
	err = yaml.Unmarshal(data, config)
	if err != nil {
		return nil, err
	}

	// Set defaults if not specified
	if config.OpenAI.Model == "" {
		config.OpenAI.Model = "whisper-1"
	}
	if config.RecordingDurationSec == 0 {
		config.RecordingDurationSec = 5.0
	}
	if config.Audio.SampleRate == 0 {
		config.Audio.SampleRate = 16000
	}
	if config.Audio.Channels == 0 {
		config.Audio.Channels = 1
	}

	return config, nil
}
