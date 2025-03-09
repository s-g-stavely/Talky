package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"mime/multipart"
	"net/http"
	"os"
)

// WhisperClient handles communication with the OpenAI Whisper API
type WhisperClient struct {
	apiKey string
	model  string
}

// NewWhisperClient creates a new Whisper API client
func NewWhisperClient(apiKey string) *WhisperClient {
	return &WhisperClient{
		apiKey: apiKey,
		model:  "whisper-1",
	}
}

// TranscribeResponse represents the response from the Whisper API
type TranscribeResponse struct {
	Text string `json:"text"`
}

// Transcribe sends the audio data to the Whisper API and returns the transcription
func (c *WhisperClient) Transcribe(audioData []byte) (string, error) {
	// Save audio to a temporary file
	tempFile, err := os.CreateTemp("", "whisper-audio-*.wav")
	if err != nil {
		return "", err
	}
	defer os.Remove(tempFile.Name())
	defer tempFile.Close()

	if _, err = tempFile.Write(audioData); err != nil {
		return "", err
	}
	tempFile.Close() // Close to ensure all data is written

	// Prepare the request
	body := &bytes.Buffer{}
	writer := multipart.NewWriter(body)

	// Add the model field
	if err = writer.WriteField("model", c.model); err != nil {
		return "", err
	}

	// Add the audio file
	file, err := os.Open(tempFile.Name())
	if err != nil {
		return "", err
	}
	defer file.Close()

	part, err := writer.CreateFormFile("file", "audio.wav")
	if err != nil {
		return "", err
	}

	if _, err = io.Copy(part, file); err != nil {
		return "", err
	}

	// Close the writer to set the content type
	if err = writer.Close(); err != nil {
		return "", err
	}

	// Create the HTTP request
	req, err := http.NewRequest("POST", "https://api.openai.com/v1/audio/transcriptions", body)
	if err != nil {
		return "", err
	}

	req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", c.apiKey))
	req.Header.Set("Content-Type", writer.FormDataContentType())

	// Send the request
	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	// Check for errors
	if resp.StatusCode != http.StatusOK {
		responseBody, _ := io.ReadAll(resp.Body)
		return "", fmt.Errorf("API error (status %d): %s", resp.StatusCode, string(responseBody))
	}

	// Parse the response
	var result TranscribeResponse
	if err = json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return "", err
	}

	return result.Text, nil
}
