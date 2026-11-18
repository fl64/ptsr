# Proposal

## Why

Change 92 gives the SMC-PAD a command panel, but it does not prove that the owner can finish a session without the keyboard. The current panel ignores most cells in project and folder pickers and in Help. Some settings and keyboard actions have no shipped controller route.

## What Changes

- Record a controller route and press count for every story in `docs/USER-STORIES.md`. Separate first panel entry from later use. Mark stories that only describe a display or another device.
- Verify the shipped keyboard map, SMC-PAD profile and panel map in both directions for each screen and mode. Exempt only text entry. Require a visible badge for each mode.
- Bind `Alt+U` to the transport toggle on every screen. Keep the current Space preview actions on Files and Chop.
- Bind `Alt+Left` and `Alt+Right` to silent pad selection on Pads. Bind `Ctrl+Alt+Up` and `Ctrl+Alt+Down` to exact 1 dB Chop threshold steps. Keep the existing Chop slice-bound keys and fixed `H` thresholds.
- Add controller selection and confirmation in the project and folder pickers. Add a relative knob route for settings values. Make Help navigation usable after its panel cell opens it.
- Keep every non-text action within two presses from a visible panel, apart from knob turns and target pad hits. Add a page selection route for context pages beyond the first.
- Add an optional MIDI setting for USB SMC-PAD. When enabled, a detected PAD BANK press opens the command panel before the next pad hit.
- Keep question priority, first-hit panel entry, keyboard keys, other devices and held pad releases.

This change SHALL NOT add Redo, Panic, date-named projects, pad lights, a host SHIFT shortcut, or a new sound feature. It SHALL NOT change the accepted rule that the splash ends after sound starts. The SMC-PAD Exit cell SHALL ask for confirmation before quit. `app.quit` SHALL stay unavailable for MIDI learn.

## Capabilities

### New Capabilities

- `controller-reach`: complete, measured paths from keyboard and SMC-PAD through current screens and modes.

### Modified Capabilities

- `keyboard`: add a shared transport key, silent pad selection keys and exact Chop threshold keys.

## Impact

The change affects input metadata, the SMC-PAD profile, panel configuration, modal dispatch, help, settings, documentation and recorded-input tests. It does not change the audio callback or project file format.

## How to verify

### Keyboard

Start an empty project. Load and chop a sample, assign slices, edit a pad, change FX and settings, save and switch projects. Verify that the current keys and modal questions keep their results.
On Files and Chop, press `Alt+U` to start and stop transport. Press Space to check the existing preview action. Repeat `Alt+U` with the Russian layout.
On Pads, use `Alt+Left` and `Alt+Right` to select without sound. On Chop and Chop edit, use `Ctrl+Alt+Up` and `Ctrl+Alt+Down` to step the threshold by 1 dB. Check both bounds and Undo.

### Controller

Repeat the session with the SMC-PAD and no keyboard text entry. Use the panel, both pad banks, pickers, settings and Help. Read the visible action and press count at each step. Verify that P7 and P8 answer only a question and that a held Gate pad releases after a modal change.
On Chop and Chop edit, turn encoder 3 in both directions. Compare its threshold, cuts and Undo result with the keyboard steps.
In F12, open MIDI and set `SMC-PAD PAD BANK` to `Open panel`. Go to Chop. Tap PAD BANK from the playing bank. Check that the panel appears before a pad hit. Press the cell for the intended action. Set the option to `Off` and check first-hit reveal again.

### Sound

This change adds no sound algorithm or new render. Listen to `renders/92-command-panel.wav` for the existing transport reference and to a loaded Gate pad while changing screens, pages and modes.
