# Proposal

## Why

While A1 plays, the selected pad flashes on each of its hits. A flash makes the frame and the fill white, so the cyan frame of the selected pad goes. The owner cannot see which pad is selected. A marked pad has the same problem: its yellow frame hides the cyan frame.

## What Changes

- The selected pad keeps its cyan frame while it flashes, while it is held and while it is marked.
- A flash fills the inside of the selected pad with white. The frame stays on the dark panel.
- A marked and selected pad keeps its dark yellow fill inside the cyan frame.
- Other pads flash as before.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pad-playback`: the selected pad keeps its frame over the hit and mark colors.

## Impact

The pad grid drawing in `crates/tui` only. No sound, input or saved-format change.

## Out of scope

- A double or thick frame. The box glyphs `═` and `━` have East Asian Width A and take two cells in some terminals.
- A pad in hand keeps its magenta frame.

## How to check

1. Load a pattern on A1 that hits the selected pad. Press Space.
2. Check that the selected pad keeps its cyan frame on each hit, and only its inside flashes white.
3. Turn marking on and mark the selected pad. Check the cyan frame around the yellow fill.
