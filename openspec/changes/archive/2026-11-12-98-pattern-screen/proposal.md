# Proposal

## Why

The owner tried recording A1 and reported that recording looks acceptable, but erasure is unclear.
The owner could not see the recorded rhythm, its position in the bar or the banks that play.
This change makes the rhythm visible on Pads (F2). A step editor for targeted removal is a separate later change.

## What Changes

- Show three event rows below the live output wave on Pads: a word header, an all-pads row and the selected pad's row.
- Show as many whole bars as the width holds, with symmetric beat groups, alternate beat shading and bar lines.
- Show the playback position as a centered light plate from the audio position, not from a UI timer.
- Support straight and triplet views in order of cell size, without changing recorded timing or playback quantize.
- Select the view with Alt+Up and Alt+Down, or with learned MIDI buttons.
- Underline banks with A1 hits in the bank row, and light closed-bank pads at their playback step.
- Explain the marks in F1. Keep saving in format 5. Do not save the view division.

The owner approved continuation of 98 alongside unfinished 97 on branch `97-pattern-record`.
This is an explicit exception to separate change branches. Keep the work distinguishable from 96 and 97.

## Capabilities

### New Capabilities

None. Change 96 introduces `patterns`. This change adds view requirements to that capability.

### Modified Capabilities

- `patterns`: independent view grids, event identity in cells and width-based pages.
- `keyboard`: view keys on Pads and their owner input rules.
- `midi-learn`: learnable view buttons.
- `screen-layout`: event rows, beat groups, playback plate, bank row, overlays and the offline catalogue.
- `screen-help`: view keys in the help line and the meaning of the marks in F1.

## Impact

The implementation affects App pattern projection, shared input defaults, help and terminal views.
Use the existing pattern compiler, stable recording-row identities and audio transport snapshots.
Do not change sound algorithms, saved data, dependencies, tempo rules or callback workload and limits.
The output wave remains live output. It is not the selected pad's sample waveform.

The first proposal also contained a step editor (`STEP EDIT`, targeted removal and its Undo).
After the owner checked the view on 2026-11-11, the editor moved to the deferred draft [98a step edit](../../../docs/plans/98a-step-edit.md).
That draft does not authorize code. A new change must accept it first.

## Out of scope

- The step editor: edit mode, edit cursor, targeted removal, event selection and removal Undo.
- New event insertion, velocity editing and Gate-length editing.
- Manual hits of closed banks in the bank row.
- Meters other than 4/4, swing, arbitrary tuplets and dotted view divisions.
- New SMC-PAD factory mappings, hardware acceptance, sound changes and saved-format migration.

## How to verify

### Keyboard

1. Use a disposable project. Record a two-bar A1 with hits on bank 1 and bank 3.
2. Open F2 on bank 1 and press Space.
3. Check the word header, the all-pads row, the pad row and the moving light plate.
4. Press Alt+Up to 1/64 and Alt+Down to 1/4. Check that the view stops at each end and the sound does not change.
5. Check that the bank row underlines numbers 1 and 3 and lights bank 3 pads at their steps.
6. Open F1 and read the strip legend.

### Controller

1. In F12 MIDI, learn `pattern.view_next` and `pattern.view_prev` on two buttons. Press Enter after each binding.
2. On F2, press the buttons. Check the same view changes as with Alt+Up and Alt+Down.

These procedures use learned controls, not claimed SMC-PAD factory support.

### Terminal views

Inspect the catalogue at 80x26 and 120x40 in `renders/screens/pads-pattern-view-*.ansi`.
The change adds no sound and no new WAV.
