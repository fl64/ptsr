# Proposal

## Why

A UI review of 96–98h on the built program found text that does not fit or does not say the right thing:

- The pad row label of the A1 strip shows `pad…`, not `pad Q`.
- At 80x26, pattern controls cut `next hits` and the Enter hint. The key line says that Enter closes, but on Clear A1 Enter asks.
- Counts say `1 hits`.
- Ctrl+Z after an erasure says `undo: pattern pass on A1`, also when no pass ran.
- In the pad edit at 80x26, the A1 help group takes the help row before the edit keys.
- F1 wraps inside words.

## What Changes

- The pad row label shows `pad` and the key whole.
- Pattern controls put a row hint on its own line when it does not fit beside the value. The key line names Enter as run and close.
- Counts use `1 hit` and `N hits`.
- Undo of a pattern step names it: `undo: erase pad Q on A1`, `undo: clear on A1`, `undo: pattern pass on A1`.
- In the pad edit, the edit keys come before the A1 group in the help row.
- F1 wraps at spaces. A word wider than its column still wraps inside the word.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `screen-layout`: whole pad row label, hints in a narrow pattern controls column.
- `screen-help`: counts, undo names, help order in the pad edit, F1 word wrap.

## Impact

Drawing and texts only. No sound, input or saved-format change.

## Out of scope

- Recording checks that need audio. The owner checks them by hand.

## How to check

1. At 80x26, select pad Q with A1 hits: the strip row says `pad Q`.
2. Alt+N at 80x26: select quantize and clear A1. The hints show whole.
3. Erase a pad with one hit: `1 hit`. Press Ctrl+Z: `undo: erase pad … on A1`.
4. Open the pad edit at 80x26: the edit keys show first.
5. Open F1 at 80x26: no word breaks inside a word.
