# Proposal

## Why

The owner asked for one look and one behavior for every edit element (`docs/plans/VISUAL.md`, section 2.9). The pad edit, the A1 pattern controls and F4 share the edit frame. The Chop slice edit shows only a badge and one text line in the right column.

## What Changes

- While the slice edit is open, the Chop right column shows an edit frame titled `EDIT · BOUNDS`, as the pad edit.
- Inside, a block line names the slice: `─ SLICE 4 ─`. Rows show `start`, `end`, `length` and `cursor`.
- The selected bound (`start` or `end`, I/K) is bright with `▸`. `length` and `cursor` are dim: they show values, the owner does not select them.
- Closed, the right column stays as it is now.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `chopping`: the slice edit uses the pad edit frame.
- `screen-layout`: the slice line of 98d shows only with the slice edit closed.

## Impact

Drawing of the Chop right column and the values the App gives it. No sound, input or saved-format change.

## Out of scope

- Settings (98g).

## How to check

1. Open Chop at 120x40 and 80x26 and press Enter for the slice edit.
2. Check the frame `EDIT · BOUNDS` with `start` bright. Press K: `end` is bright. Press J/L: the bound moves and its value changes.
3. Press Esc: the right column shows the slice line again.
