# Talky

Talky lets you type using your voice, on Windows, Mac and Linux (X11 only due to hotkey library TODO). It uses OpenAI's speech-to-text model, Whisper, to convert your speech into text, which is pasted into whatever app you're using. 

## Installation

Download the latest release from https://github.com/s-g-stavely/Talky/releases for your OS. Extract the application anywhere and run it.

TODO: more details, how do you run things on mac

## Configuration

TODO config file, running locally
https://huggingface.co/Mozilla/whisperfile/blob/main/whisper-tiny.en.llamafile


## TODOs
run locally without webserver?
tell it not to output silence
chloe saw the end PCM bug again
create release package
put in tray
fix bug with not picking up first press
make hotkey configurable
make exiting via ctrl c better

it's maybe weird to be using winit when there's no GUI