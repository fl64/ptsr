## Why

A copy, swap and file can wait at the same time. The keyboard and SMC-PAD can
then put different items on the same pad.

## What Changes

- Keep one pending copy, swap or file.
- Replace the earlier item when the next pickup succeeds.
- Keep the earlier item when a pickup fails.
- Use the same item for keyboard and controller targets.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- pad-assignment: choose one item for placement.
- screen-layout: show the badge for that item.

## How to check

### Keyboard

1. Select a loaded pad on Pads. Press Ctrl+C. Press Ctrl+X.
2. Check the SWAP label. Hit an empty pad. Check that the sound moved.
3. Undo. Select the loaded pad. Press Ctrl+X. Press Ctrl+C.
4. Check the COPY label. Hit an empty pad. Check that both pads have the sound.
5. Take another copy. Open Files. Press Enter on a file. Check the IN HAND label.
6. Hit an empty pad. Check that it receives the file.

### SMC-PAD

1. On Pads, hold PAD BANK. Press P6 Copy. Release PAD BANK.
2. Hold PAD BANK. Press P3 Menu, then P9 Swap. Release PAD BANK.
3. Check the SWAP label. Hit an empty target pad. Hit it again to hear the sound.
4. Repeat the file pickup with Files P6 Take. Use a controller pad as the target.

### Sound

This change needs no new WAV. Listen to the target pad after placement.
