## Why

The STEP encoder moves the view to another time cell. The overview shows
the new number but does not mark its cell. The number starts at one in
each bar. On an 80x26 terminal, the long title can hide the total step
count or the recording quantize.

## What Changes

- Show the selected bar, total bar count, selected step, and total step count.
- Show the view grid and recording quantize on their own line.
- Mark the selected time column in the overview and Step Edit. Mark
  playback separately.
- Use the same selected step after entry to Step Edit and after Back.
- Show the pad name, position, and result when a pad adds or removes a hit.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- patterns: read and set the same time position in the overview and Step Edit.

## How to check

Use a two-bar A1 with the 1/16 view. Repeat at 80x26 and 120x40.

### Keyboard

1. Open A1 with F5. Use J/L or Left/Right to move from step 16 to 17.
2. Check the bar, total steps, view grid, and recording quantize.
3. Press F7. Hit a pad. Press Ctrl+Z. Press Esc.
4. Check that the cursor keeps step 17.

### SMC-PAD

1. Open A1 from Screens P16. Turn STEP from step 16 to 17.
2. Check the bar, total steps, view grid, and recording quantize.
3. Hold PAD BANK and press P6 Edit Steps. Release PAD BANK. Hit a pad.
4. Hold PAD BANK and press P2 Undo. Press P1 Back. Release PAD BANK.
5. Check that the cursor keeps step 17.

When the hold shortcut is off, use the second pad bank for commands.
The first command pad press shows a hidden panel.

### Sound

The change needs no new WAV. The screen and navigation do not change
the sound or the timing of saved hits.
