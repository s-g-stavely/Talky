package main

import (
	"fmt"
	"log"
	"os"
	"os/signal"
	"path/filepath"
	"syscall"

	"github.com/getlantern/systray"
	hook "github.com/robotn/gohook"
)

func main() {
	add()

	// Load configuration
	config, err := LoadConfig(filepath.Join(".", "config.yaml"))
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

		hook.Register(hook.KeyUp, []string{"w"}, func(e hook.Event) {
			fmt.Println("Hotkey pressed! Starting recording...")

			// Create audio recorder
			recorder := NewAudioRecorder(
				config.Audio.SampleRate,
				config.Audio.Channels,
				config.Audio.BitDepth,
			)

			// Record to a file directly
			recordingFilePath := "recording.wav"
			err := recorder.RecordAndSaveWAV(recordingFilePath, config.RecordingDurationSec)
			if err != nil {
				log.Printf("Failed to record audio: %v", err)
				return
			}

			// Transcribe with Whisper API using the file directly
			client := NewWhisperClient(config.OpenAI.APIKey)
			transcription, err := client.TranscribeFile(recordingFilePath)
			if err != nil {
				log.Printf("Failed to transcribe audio: %v", err)
				return
			}

			log.Printf("Transcription: %s", transcription)

			// Insert text at cursor position
			err = InsertTextAtCursor(transcription)
			if err != nil {
				log.Printf("Failed to insert text: %v", err)
				return
			}

			// Optionally remove the recording file when done
			// os.Remove(recordingFilePath)
		})

		s := hook.Start()
		<-hook.Process(s)

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

func add() {

	fmt.Println("--- Please press w---")
	hook.Register(hook.KeyDown, []string{"w"}, func(e hook.Event) {
		fmt.Println("keyDown: ", "w")
	})
}
