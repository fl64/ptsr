# 02c — MIDI learn

## Why

Profiles in files are hard to edit by hand: you have to know which CC or note a control sends and where it sits on the device. It is faster to pick an action on screen and just move the control you want for it.

## What changes

- A MIDI tab on the settings screen (F12): every action of the program, grouped as GLOBAL (any screen), PADS and SETTINGS, each with its current assignment; the list scrolls. Screens added later bring their own group. Quit is not on the list, so a stray press cannot close the program.
- Enter on an action waits for a control; the control you move next is assigned and saved at once. Esc cancels, Backspace clears.
- One control drives one action within a group: assigning it moves it away from its old action there. A screen group wins over GLOBAL on its screen.
- Learned assignments are saved in `~/.config/arasaka-404/config.toml` and override the device profile; the profile file in the repository stays as it is.

## Out of scope

Assignments for screens that do not exist yet (chop, files); knob type and sensitivity settings; MIDI output.

## How to verify

1. `task run`, F12, Tab to MIDI.
2. Select "pad volume", Enter, turn the knob you like — the row shows it.
3. Select "bank next", Enter, press a button — the row shows it.
4. Esc to the pads screen: the knob changes the pad volume, the button switches banks.
5. Restart the program — the assignments are still there.
