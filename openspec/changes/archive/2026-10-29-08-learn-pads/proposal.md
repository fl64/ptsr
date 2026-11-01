# 08 — Pad hits in MIDI learn

## Why

The MIDI tab lists banks, pad mode and pad volume, but not the pad hits themselves: a controller button or pad can drive everything except the one thing a sampler is for. Today the pads sound only from the keyboard block and from device profiles that know the device in advance. The owner wants to learn pad hits on any controller.

## What Changes

- **16 new actions "pad 1" … "pad 16"** in the GLOBAL group of the MIDI tab. They hit the pads of the current bank, so they work on every screen and on every bank.
- A learned button, note or controller pad hits the pad with **velocity**: a note carries its velocity, a CC button uses its value.
- **Gate mode works from MIDI**: the learned control's release (note off, CC value 0) stops a gate pad, without a click.
- Pad hits are press actions, so a knob is not offered for them in learn (like every button action, though, a knob that sends a press-like CC can be captured — raw MIDI does not tell a knob from a button).

## Capabilities

Modified: `midi-learn` (the action list includes pad hits), `pad-playback` (pad hits from a learned control: velocity, gate, every screen).

## Out of scope

- Per-bank bindings (a control always hits pad N of the current bank).
- Renaming pads, 10 banks, new controller profiles.

## How to verify

By hand:

1. F12 → MIDI → GLOBAL → "pad 1", press Enter, hit a button on the controller. The row names it.
2. On any screen (pads, files, chop, settings) press that button: pad 1 of the current bank plays. Switch the bank — the same button plays pad 1 of the new bank.
3. Learn another button to "pad 2", put a gate-mode pad on pad 2, hold the button — the sound holds — release it — the sound stops without a click.
4. Hit the button lightly/softly if it sends velocity: the volume follows.
5. Restart: the learned pad hits still work. Backspace clears one, the profile assignment applies again.

Listen for: the hit answers at once, no click on the gate release.
