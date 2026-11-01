# 28 — The MIDI learn list follows the layers, folds and learns pads in a row

## Why

The owner (2026-10-30, a screenshot of the F12 MIDI tab): "a solid sheet". About 100 rows
in four open groups, 24 of them only for pads 1–16 and banks 1–8. The edit layer (the
page knobs, the parameter pick, leaving the edit) is not in the list at all: it lives only
in the device profile, so one cannot see or learn that the same knob does another thing
inside the edit — "I enter the mode and use the same knobs and buttons as before".

## What changes

- **Groups follow the layers**: GLOBAL, PADS, PADS · EDIT, SETTINGS, FILES, CHOP,
  CHOP · EDIT. The EDIT groups hold the page knobs 1–3, the previous and the next
  parameter and leaving the edit. A control can be learned in PADS and in PADS · EDIT for
  different actions; inside the edit the EDIT group wins, as the profile's `[edit]` layer
  does today.
- **Groups fold**: a closed group is one row, `PADS · 14 actions · 3 learned`. The group of
  the screen F12 was opened from is open, the others are closed. `→` (`L`) opens, `←`
  (`J`) closes, `Enter` on a group row opens it.
- **Pads and banks in a row**: "pads 1–16" and "banks 1–8" are one row each. `Enter` on it
  learns them one after another: every control pressed goes to the next pad (bank); `Esc`
  stops, what is learned stays. The row says how many are learned.

## Out of scope

A filter "learned only" (offered, not taken now); renaming actions; device profiles.

## How to verify

F12 from the pads screen, MIDI tab: only PADS is open, the other groups are single rows
with counts. Open PADS · EDIT, learn knob 1 of the page on the knob that is "pad volume" in
PADS: outside the edit it turns the volume, inside the edit the first parameter of the
page. "pads 1–16", `Enter`, hit 16 controller pads in turn: pads 1–16 play from them.
