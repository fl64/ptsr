# Proposal

## Why

Some controller actions need a short press and a hold on the same button. At present, a MIDI button runs one action when it goes down. The owner cannot use one button for both roles or hold it to record the microphone.

## What Changes

- Add tap and hold actions for a MIDI button. A button with a hold action sends its tap when released before the hold threshold.
- Let another control used on the same device start the hold at once. A hold action receives a down event and an up event.
- Let the owner set the hold threshold from 200 to 600 ms. The default is 300 ms.
- Let MIDI learn keep a tap and a hold action on one button. Existing saved action strings become tap actions without changing their timing.
- End held actions when a device disconnects. Keep direct pad releases and input order.
- Keep plain button bindings immediate. The STOP role must remain immediate when it uses a plain binding.

This change SHALL add no SMC-PAD device profile, command layer, dependency, or sound algorithm.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `input-devices`: timed button gestures and profile bindings.
- `midi-learn`: tap and hold assignment and saved-binding migration.
- `audio-settings`: the hold threshold on the MIDI settings tab.

## Impact

The change affects MIDI profile parsing, dispatch, learned bindings, and the MIDI settings tab. It does not change project files or the audio callback.

## Out of scope

The measured SMC-PAD profile is change 91. The command panel and layers are change 92. Relative encoders are change 90. No unmeasured hardware button gets a factory action here.

## How to check

### Keyboard

1. Press the existing transport, metronome, and microphone keys. Check that their press and release behavior stays the same.
2. Open F12 MIDI. Change the hold threshold and restart. Check that the selected value remains.

### Controller

1. With a test profile, tap PLAY. Check that transport changes only when you release PLAY. Hold PLAY. Check that the metronome changes once at the threshold and that transport does not change.
2. Hold RECORD, then release it. Check that microphone recording starts and stops once. Press another control during the hold and check that recording starts before that control acts.
3. Press a plain button binding. Check that it acts on press. Disconnect during a hold. Check that its up action runs and that reconnecting starts with no held action.

### Sound

This change adds no sound process or render. Listen for a clean end to microphone recording and a held Gate pad after device disconnect.
