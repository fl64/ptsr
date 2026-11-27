## Why

A folder picker can catch a Gate release. A question can catch the release of a held microphone control. Releasing Alt before R can also leave the microphone on.

## What Changes

- Release the keyboard Gate that started before a folder picker opened.
- Finish a held microphone take when its key or SMC-PAD RECORD button comes up over an open question.
- Stop a keyboard hold in either release order.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- keyboard: deliver an earlier Gate release through the folder picker.
- recording: deliver a microphone release in either order and through a question.

## How to check

### Keyboard

1. Hold a Gate pad. Open F12 and go to Paths. Open a folder picker. Release the pad and listen for silence.
2. On Pads, hold Alt+R. Open a pad clear question with Backspace. Release R, then Alt. Check that the question stays open and the take stops.
3. Repeat the second check with Alt first. In a terminal without key releases, use two Alt+R presses.

### SMC-PAD

1. Hold RECORD until microphone recording starts. Open a clear question.
2. Release RECORD. Check that the question stays open and the take stops.

### Sound

Listen for the end of the Gate and play the WAV made by the microphone check.
