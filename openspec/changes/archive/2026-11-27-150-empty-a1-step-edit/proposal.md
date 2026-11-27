## Why

F7 opens Step Edit on an empty A1. The P6 cell stays grey in the same state.

## What Changes

- Let P6 open Step Edit on an existing empty A1.
- Use the same entry checks for P6 and F7.
- Show why the current state prevents entry.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- command-panel: show the Step Edit action for the current A1.

## How to check

### Keyboard

1. Open A1 with F5. Create an empty A1 in Setup. Close Setup.
2. Press F7. Hit a pad. Check the first step. Press Ctrl+Z.

### SMC-PAD

1. Open A1 with Screens P16. Create an empty A1 in Setup. Close Setup.
2. Hold PAD BANK. Check that P6 edit steps is active. Press P6.
3. Release PAD BANK. Hit a pad. Check the first step.
4. Hold PAD BANK. Press P2 Undo. Check that the step clears.

### Sound

This change needs no new WAV. Step Edit stays silent during entry and edit.
