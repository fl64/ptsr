# 02b — MIDI controllers

## Why

Play and turn parameters from controllers — X-Session Pro now, SMC-PAD later, both at once — and a change of controller should not need changes to the program.

## What changes

- MIDI: several devices at once, hot plugging.
- Device profiles in TOML: hardware → logical actions; a default profile; a profile with an error does not crash the program.
- Soft takeover for absolute knobs and faders.
- An input debug tab on the settings screen (F12).
- An X-Session Pro profile (numbers are read with `aseqdump` or the input debug tab).

## Out of scope

MIDI learn in the UI (profiles in files only), MIDI output, clock.

## How to verify

Connect the X-Session Pro, open F12 → input debug, turn a knob — you see the CC number and the action. Play pads from the keyboard and turn the pad volume with a knob at the same time. Pull out the controller — the sound does not stop; plug it back in — it works again.
