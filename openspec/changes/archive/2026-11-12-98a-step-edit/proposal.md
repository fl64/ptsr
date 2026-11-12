# Proposal

## Why

The A1 overview shows the recorded rhythm across all banks, but it cannot correct one unwanted hit. Whole-pad erasure removes too much when a take has several hits on one pad.

## What Changes

- Add a step edit mode to the A1 overview. F5 keeps its current screen role. The proposed F7 key enters or leaves edit mode.
- Let the owner select a time cell and remove one existing hit with its pad. An empty cell does not add a hit.
- Show the edit cursor and the chosen event separately from the moving playback position. Keep the current bank and sample names visible.
- Keep edit pad presses silent through keyboard and raw MIDI. Preserve releases of pads that sounded before edit entry.
- Give each successful removal one Undo step. Save the corrected raw pattern in the existing format.
- Add learnable MIDI buttons, effective help, offline screen states and a generated-sound render of removal and Undo.

## Capabilities

### New Capabilities

None. The existing A1 pattern capability owns the correction.

### Modified Capabilities

- `patterns`: selected time cells, one-event removal, Undo and save behavior.
- `keyboard`: editor entry and contextual step and event controls.
- `input-devices`: silent edit pad routing and original release ownership.
- `midi-learn`: learnable edit, step and event buttons.
- `screen-layout`: the edit cursor, selected event and overview states.
- `screen-help`: effective editor controls, feedback and refusals.

## Impact

The change affects the A1 view, input routing, history, help and terminal drawing. It uses the existing pattern compiler and audio publication path. It adds no dependency, saved format or sound algorithm.

## Out of scope

- Adding, moving or changing the velocity or length of a hit.
- New factory MIDI mappings or a claim of physical SMC-PAD acceptance.
- A new meter, pattern slot or audio effect.

## How to verify

### Keyboard

1. Use a copy of a project with A1 hits in two banks. Press F5, then F7.
2. Select a time cell and press an occupied pad. See one hit disappear without a pad sound.
3. Press Ctrl+Z. See the same hit return while playback keeps its place.
4. Try an empty cell, a cell with several events and F1. No unrelated hit must disappear.

### Controller

1. Learn the edit, step and event actions on MIDI buttons in F12.
2. Use these buttons and a physical pad to remove one hit. Hear a Gate from before edit entry release.

### Sound and views

Listen to the new generated-sound render of an A1 hit, its removal and Undo. Other pads and held tails must stay unchanged. Inspect the overview at 80x26 and 120x40.
