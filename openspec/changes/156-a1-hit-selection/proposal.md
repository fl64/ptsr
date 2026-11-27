## Why

The keyboard can select one of several hits in a Step Edit cell. The shipped SMC-PAD map has no control for this.

## What Changes

- Use device pads 14 and 15 for the previous and next hit in Step Edit.
- Show the number and velocity of the selected hit. Keep the controls dim in the overview.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- command-panel: select an A1 hit from the shipped panel.

## How to check

### Keyboard

1. Open a cell with two hits on the same pad. Use I and K to change the hit.

### SMC-PAD

1. Open Step Edit. Hit the pad with two hits in one cell to select the first hit.
2. Hold PAD BANK. Press P15. Check the second hit number and its velocity.
3. Release PAD BANK. Hit the same pad to remove that hit. Check the first hit.
4. Hold PAD BANK. Press P2 Undo. Check both hits.

### Sound

This change needs no new WAV. The selection and edit stay silent.
