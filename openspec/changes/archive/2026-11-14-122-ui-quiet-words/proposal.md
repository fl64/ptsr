# Proposal

## Why

Empty pad cells repeat the word `empty` and draw flat waves. Several short labels do not explain their meaning. The clip light also occupies a row when there is no clip.

## What Changes

- Show a muted `—` in an empty pad cell. Do not draw a flat pad wave. Keep a meaningful corner mark and the test tone in the pad edit.
- Use plain names in the changed edit-scope help, sample-source text, pattern help, and pattern keys. Keep Pitch Gate state and restrictions visible in the pad panel and F1.
- Keep the ten-cell playback slots. Leave the idle PLAY slot blank. Keep active PLAY and LISTEN feedback.
- Hide the idle clip label. Show `CLIP` with the elapsed session time of the latest clipped block while the existing clip latch is on.
- Shorten long labels at word boundaries where a whole word fits. Keep graphemes and safety text whole.

This change SHALL NOT change keys, logical actions, sound, saved data, or the audio callback.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pad-playback`: empty cells, pad waves, and the visible source name.
- `screen-layout`: visible labels, playback slots, Pitch Gate summary, clipping feedback, and text clipping.
- `screen-help`: edit scope, pattern help, and visible names.

## Impact

The change affects terminal drawing and the UI-side meter state. It adds no dependency or data migration. The sound stays unchanged.

## Out of scope

Body-row redistribution belongs to the later `screen-layouts` change. This change keeps fixed screen rows and input routes.
The legacy `No A1` strip text moves to `pattern-screen`. The `PG on` and `scope pad 1`
panel text moves to `pads-context-lane`. The `CLOCK` help group moves to `compact-help`.
The owner accepted change 122 with these four labels open on 2026-11-14.

## How to check

### Keyboard

1. Open F2 with an empty bank. Check the quiet pad cells, the blank idle PLAY slot, and the hidden idle clip label.
2. Select an empty pad. Open its edit and check that its test tone view remains. Load a file and check `whole file` and `slice 3` source labels.
3. Mark three pads, then clear the marks. Check `edit: marked (3)` and `edit: this pad`. Check the bank scope.
4. Set Pitch Gate on a pad at zero and positive pitch. Check the compact text, panel, and F1.
5. Open Pads, Chop, Files, FX, Settings, the A1 overview, and F1 at 80x26 and 120x40. Check labels and long names.

### Controller

1. Use learned screen actions to open Pads, Chop, Files, FX, Settings, and F1. Check the same labels.
2. Use a learned pad and value control to select and edit a pad. Check that input and sound stay the same.

### Views and sound

Review `pads-normal`, `pads-pattern-empty`, `pads-pattern-overview-absent`, `chop-normal`, `files-normal`, `fx-normal`, `settings-normal`, and `pads-help` at both supported sizes. New `pads-quiet-empty`, `pads-clip-event`, and `pads-long-labels` catalogue entries will cover changed states at both sizes. Listen to the existing local `renders/82-shared-fx-screen.wav` and `renders/97-pattern-record.wav`. This change needs no new WAV because sound stays unchanged.
