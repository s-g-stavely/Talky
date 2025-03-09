package main

import (
	"github.com/atotto/clipboard"
	"github.com/go-vgo/robotgo"
)

// InsertTextAtCursor inserts the given text at the current cursor position
func InsertTextAtCursor(text string) error {
	// Save current clipboard content
	oldClipboard, err := clipboard.ReadAll()
	if err != nil {
		return err
	}

	// Copy the transcribed text to clipboard
	err = clipboard.WriteAll(text)
	if err != nil {
		return err
	}

	// Simulate paste keyboard shortcut
	robotgo.KeyTap("v", "cmd") // For macOS
	// For Windows, use: robotgo.KeyTap("v", "ctrl")

	// Small delay to ensure paste completes
	robotgo.MilliSleep(100)

	// Restore the original clipboard content
	return clipboard.WriteAll(oldClipboard)
}
