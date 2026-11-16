# Proposal

## Why

An absolute MIDI knob sets a position. A controller with an endless encoder sends direction and step size instead. The current profile loader ignores controls marked `type = "relative"`. The owner cannot use such an encoder to move a value or a list.

## What Changes

- Accept three relative CC encodings in device profiles and turn each message into signed steps. Reject an unknown encoding with the profile file and control name.
- Apply relative steps to parameters and lists through the existing screen actions. Use the same small value step as a keyboard tap when acceleration is off.
- Increase step size during rapid turns. A held, assigned fine modifier disables acceleration for its device.
- Let relative knobs select files, projects, settings rows, edit pages and banks. Keep controller commands in receipt order before later pad hits.
- Keep absolute knobs, saved assignments, keyboard controls and pad release routes unchanged.

This change SHALL add no SMC-PAD profile, command panel, dependency or sound algorithm.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `input-devices`: relative profile controls, decoding, acceleration and fine modifier.
- `editing`: step sizes for parameters and page movement.
- `sample-loading`: relative file-list selection and preview.
- `projects`: relative project-picker selection.
- `audio-settings`: relative settings-row selection.

## Impact

The change affects MIDI profile parsing, dispatch, parameter steps and selection in the UI. It does not change the audio callback or project-file format.

## Out of scope

Change 91 measures SMC-PAD messages and adds its profile. Change 92 adds the command panel and its screen-specific knob roles. This change does not assume that the SMC-PAD uses a relative encoding or that SHIFT sends MIDI.

## How to check

### Keyboard

1. Use J and L to change a pad's volume, pitch, envelope and FX values. Check the current small steps.
2. Use the existing keys to move through Files, the project picker, F12 rows, edit pages and banks.

### Controller

1. Use a test profile with one relative encoder. Turn it slowly in both directions. Check one step per detent and no pickup arrow.
2. Turn the encoder quickly. Check larger steps. Hold its assigned fine button and turn it again. Check small steps and a return to normal after release or disconnect.
3. Move through a file list, the project picker, settings rows, pages and banks. Check that later pad hits use the new selection or bank.
4. Turn an existing absolute knob on the X-Session Pro. Check that its jump, pickup or scale setting still applies.

### Sound

This change adds no sound process or render. Listen to file preview while scrolling and to the existing pad sound while values change.
