# Talky

Talky lets you type using your voice, on Windows, Mac and Linux (X11 only due to hotkey library TODO). It uses OpenAI's speech-to-text model, Whisper, to convert your speech into text, which is pasted into whatever app you're using. 

## Installation

Download the latest release from https://github.com/s-g-stavely/Talky/releases for your OS. Extract the application anywhere and run it.

TODO: more details, how do you run things on mac

## Configuration

TODO config file, running locally
https://huggingface.co/Mozilla/whisperfile/blob/main/whisper-tiny.en.llamafile


## Development

### Linux

`sudo apt install pkg-config libssl-dev`


## TODOs
- run locally without webserver?
- create release package
- put in tray
- make exiting via ctrl c better
- if you run two copies at once get "hotkey already registerd", fail more gracefully
- config file should handle updates with new vals
- remove log messages and make api key log clearer
- print hotkey
- it's maybe weird to be using winit when there's no GUI