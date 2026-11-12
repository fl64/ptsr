# Proposal

## Why

The owner asked for one look and one behavior for every edit element (`docs/plans/VISUAL.md`, section 2.9). The pad edit, the A1 pattern controls, F4 and the Chop slice edit share the edit frame. F12 shows its tabs as `[Audio] Input MIDI Paths` in the title line, capitalized labels and an inverse selected row.

## What Changes

- F12 draws its body in a frame titled `SETTINGS · AUDIO INPUT MIDI PATHS`. The open tab is lit, as the open page of the pad edit. The title line keeps `F12 SETTINGS` and the badges.
- Labels are lowercase: `output`, `input`, `buffer`, `wave`, `projects root`, `samples root`.
- The selected row is bright with `▸`, as in the other edit frames, in place of the inverse row. This applies to the MIDI list too.
- Explanation lines stay dim under their rows. Tab, I/K, J/L, Enter, Backspace and Esc keep their roles.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `audio-settings`: F12 uses the edit frame.

## Impact

Drawing of F12 only. No setting, input or saved-format change.

## Out of scope

- The F1 controller map (later change).

## How to check

1. Open F12 at 120x40 and 80x26.
2. Check the frame with AUDIO lit. Press Tab three times: INPUT, MIDI, PATHS are lit in turn.
3. Press I/K: the selected row is bright with `▸`.
