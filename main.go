package main

import (
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/getlantern/systray"
)

func main() {
	// Initialize application
	config, err := LoadConfig("config.yaml")
	if err != nil {
		log.Fatalf("Failed to load configuration: %v", err)
	}

	// Start systray for background operation
	go systray.Run(onReady(config), onExit)

	// Wait for termination signal
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
	<-sigChan
}

func onReady(config *Config) func() {
	return func() {
		systray.SetTitle("Talky")
		systray.SetTooltip("Speech-to-text at your cursor")
		mQuit := systray.AddMenuItem("Quit", "Quit the application")

		// Set up hotkey listeners
		hotkeyManager, err := NewHotkeyManager()
		if err != nil {
			log.Fatalf("Failed to initialize hotkey manager: %v", err)
		}
		defer hotkeyManager.Close()

		// Register hotkey for triggering recording
		err = hotkeyManager.Register(config.Hotkey, func() {
			fmt.Println("Hotkey pressed! Starting recording...")

			// Start audio recording
			audioData, err := RecordAudio(config.RecordingDurationSec)
			if err != nil {
				log.Printf("Failed to record audio: %v", err)
				return
			}

			// Transcribe with Whisper API
			client := NewWhisperClient(config.OpenAI.APIKey)
			transcription, err := client.Transcribe(audioData)
			if err != nil {
				log.Printf("Failed to transcribe audio: %v", err)
				return
			}

			// Insert text at cursor position
			err = InsertTextAtCursor(transcription)
			if err != nil {
				log.Printf("Failed to insert text: %v", err)
				return
			}
		})

		if err != nil {
			log.Fatalf("Failed to register hotkey: %v", err)
		}

		// Wait for quit action
		go func() {
			<-mQuit.ClickedCh
			systray.Quit()
		}()
	}
}

func onExit() {
	// Clean up resources if needed
}
