package main

import (
	"bytes"
	"time"

	"github.com/gordonklaus/portaudio"
)

// RecordAudio records audio from the default input device for the specified duration
func RecordAudio(durationSec float64) ([]byte, error) {
	// Initialize PortAudio
	if err := portaudio.Initialize(); err != nil {
		return nil, err
	}
	defer portaudio.Terminate()

	// Default configuration
	sampleRate := 16000 // 16 kHz, which works well with Whisper
	channels := 1       // Mono
	frameSize := 1024   // Number of samples per frame

	// Open default input stream
	inputBuffer := make([]int16, frameSize)
	audioBuffer := new(bytes.Buffer)

	stream, err := portaudio.OpenDefaultStream(channels, 0, float64(sampleRate), frameSize, inputBuffer)
	if err != nil {
		return nil, err
	}
	defer stream.Close()

	// Start recording
	if err = stream.Start(); err != nil {
		return nil, err
	}

	// Record for the specified duration
	recordDuration := time.Duration(durationSec * float64(time.Second))
	recordEndTime := time.Now().Add(recordDuration)

	for time.Now().Before(recordEndTime) {
		if err = stream.Read(); err != nil {
			return nil, err
		}

		// Convert and write samples to buffer
		for _, sample := range inputBuffer {
			// Write the int16 sample as bytes to the buffer
			audioBuffer.WriteByte(byte(sample))
			audioBuffer.WriteByte(byte(sample >> 8))
		}
	}

	// Stop recording
	if err = stream.Stop(); err != nil {
		return nil, err
	}

	return audioBuffer.Bytes(), nil
}

// SaveWaveFile writes the recorded audio to a WAV file
// Useful for debugging or sending to the Whisper API
func SaveWaveFile(filename string, audioData []byte, sampleRate, channels int) error {
	// In a real implementation, you'd write proper WAV file headers
	// and format the audio data correctly.
	// This is a placeholder for that functionality.
	return nil
}
