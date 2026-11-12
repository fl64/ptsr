# Proposal

## Why

On Chop, the slice line in the right column wraps inside a value. At 120x40 the owner can see `start 0.750` on one row and `s · length 250 ms` on the next. The line wraps at any space, also between a number and its unit.

## What Changes

- The slice line wraps only between its parts: `slice 4`, `start 0.750 s`, `length 250 ms`, `cursor 0.000 s`. A part stays whole on one row.
- A part wider than the column still wraps, so no text goes away.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `screen-layout`: the Chop slice line keeps each value whole.

## Impact

Drawing of the Chop right column only. No sound, input or saved-format change.

## Out of scope

- The edit frame for the Chop slice edit (98f).

## How to check

1. Open Chop at 120x40 and at 80x26. Press Enter for the slice edit.
2. Check that each value of the slice line stays on one row with its unit.
