# Design 02c

## Storage

`[midi.<profile name>.<group>]` in the user config (group: `global`, `pads`, `settings`), key is the MIDI number, value is the action: `"cc 24" = "pad.volume"`. Devices without a profile use the profile name `default`. The section is written next to `[audio]`; the rest of the file is kept.

## Override rules

For a device and screen, a learned control is looked up in the screen group first, then in GLOBAL. A learned action replaces every profile assignment of the same action, so the old control stops driving it. A learned control replaces whatever the profile assigned to that control.

## Action list

One table in the `input` crate lists every action with its name, label, group and kind (button or knob). The keymap parser and the MIDI tab both use it; a test checks that every listed button action parses.

## Learning

The UI thread already receives every MIDI message for the input tab. While waiting for a control:

- a button action takes the first note-on or non-zero CC;
- a knob action (pad volume) takes the CC that moved most within 0.5 s after the first message, so touching a neighbour knob does not win. Notes are ignored for knob actions.

New assignments go to the MIDI thread through its message channel, so the next message already uses them.
