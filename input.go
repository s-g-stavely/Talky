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

	// output the text at the cursor
	robotgo.TypeStr(text)

	// Small delay to ensure paste completes
	robotgo.MilliSleep(100)

	// Restore the original clipboard content
	return clipboard.WriteAll(oldClipboard)
}
