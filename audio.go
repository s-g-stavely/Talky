package main

import (
	"bytes"
	"fmt"
	"os"
	"time"

	"github.com/go-audio/audio"
	"github.com/go-audio/wav"
	"github.com/gordonklaus/portaudio"
)

// AudioRecorder handles audio recording functionality
type AudioRecorder struct {
	SampleRate int
	Channels   int
	BitDepth   int
}

// NewAudioRecorder creates a new audio recorder with the specified settings
func NewAudioRecorder(sampleRate, channels, bitDepth int) *AudioRecorder {
	return &AudioRecorder{
		SampleRate: sampleRate,
		Channels:   channels,
		BitDepth:   bitDepth,
	}
}

// RecordAudio records audio from the default input device for the specified duration
func (ar *AudioRecorder) RecordAudio(durationSec float64) ([]int16, error) {
	// Initialize PortAudio
	if err := portaudio.Initialize(); err != nil {
		return nil, err
	}
	defer portaudio.Terminate()

	frameSize := 1024 // Number of samples per frame

	// Open default input stream
	inputBuffer := make([]int16, frameSize*ar.Channels)
	var allSamples []int16

	defaultDevice, err := portaudio.DefaultInputDevice()

	if err != nil {
		return nil, fmt.Errorf("failed to get default input device. err: %w", err)
	}

	fmt.Printf("Using default device: %s\n", defaultDevice.Name)

	stream, err := portaudio.OpenDefaultStream(ar.Channels, 0, float64(ar.SampleRate), frameSize, inputBuffer)
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

		// Append the samples to our slice
		allSamples = append(allSamples, inputBuffer...)
	}

	// Stop recording
	if err = stream.Stop(); err != nil {
		return nil, err
	}

	return allSamples, nil
}

// SaveWaveFile writes the recorded audio to a WAV file
func (ar *AudioRecorder) SaveWaveFile(filename string, samples []int16) error {
	// Create the output file
	out, err := os.Create(filename)
	if err != nil {
		return fmt.Errorf("failed to create output file: %w", err)
	}
	defer out.Close()

	// Create a new encoder
	enc := wav.NewEncoder(out, ar.SampleRate, ar.BitDepth, ar.Channels, 1) // 1 = PCM format
	defer enc.Close()

	// Convert the int16 samples to an audio.IntBuffer
	buf := &audio.IntBuffer{
		Format: &audio.Format{
			NumChannels: ar.Channels,
			SampleRate:  ar.SampleRate,
		},
		Data:           make([]int, len(samples)),
		SourceBitDepth: ar.BitDepth,
	}

	// Convert int16 samples to int for the IntBuffer
	for i, sample := range samples {
		buf.Data[i] = int(sample)
	}

	// Write the buffer to the encoder
	if err := enc.Write(buf); err != nil {
		return fmt.Errorf("failed to write audio data: %w", err)
	}

	if err = enc.Close(); err != nil {
		return err
	}

	if err = out.Close(); err != nil {
		return err
	}

	out2, err := os.Open("recording.wav")
	if err != nil {
		panic(err)
	}
	d2 := wav.NewDecoder(out2)
	d2.ReadInfo()
	fmt.Println("New file ->", d2)
	out2.Close()

	return nil
}

// RecordAndSaveWAV records audio and saves it directly to a WAV file
func (ar *AudioRecorder) RecordAndSaveWAV(filename string, durationSec float64) error {
	samples, err := ar.RecordAudio(durationSec)
	if err != nil {
		return err
	}

	return ar.SaveWaveFile(filename, samples)
}

// SeekableBuffer is a wrapper around bytes.Buffer that implements io.WriteSeeker
type SeekableBuffer struct {
	buf *bytes.Buffer
	pos int64
}

// Write implements io.Writer
func (sb *SeekableBuffer) Write(p []byte) (n int, err error) {
	n, err = sb.buf.Write(p)
	sb.pos += int64(n)
	return
}

// Seek implements io.Seeker
func (sb *SeekableBuffer) Seek(offset int64, whence int) (int64, error) {
	newPos := int64(0)

	switch whence {
	case 0: // io.SeekStart
		newPos = offset
	case 1: // io.SeekCurrent
		newPos = sb.pos + offset
	case 2: // io.SeekEnd
		newPos = int64(sb.buf.Len()) + offset
	default:
		return 0, fmt.Errorf("invalid whence value: %d", whence)
	}

	if newPos < 0 {
		return 0, fmt.Errorf("negative position")
	}

	if newPos > int64(sb.buf.Len()) {
		// If seeking beyond the end, pad with zeros
		sb.buf.Write(make([]byte, newPos-int64(sb.buf.Len())))
	}

	sb.pos = newPos
	return sb.pos, nil
}
