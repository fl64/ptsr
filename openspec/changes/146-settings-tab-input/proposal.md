## Why

The owner can change an Audio setting while Input or Paths is open.
Paths can also retain a row number that it cannot show.
The command panel covers the folder list in a small window.

## What Changes

- Limit the Settings value keys and E2 to tabs with value controls.
- Keep Input available for MIDI event inspection.
- Select a visible root when Paths opens. Keep the folder picker controls.
- Dim unavailable encoders in the strip and Controller Help.
- Keep the selected folder and its controls visible while the picker is open.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- controller-reach: limit Settings row and value controls to the active tab.

## How to check

### Keyboard

1. Open F12 Audio. Select Click level and note the percent.
2. Press Tab to open Input. Press Down and Left.
3. Press Tab twice to open Paths. Check the selected root.
4. Press Left. Press Enter. Check the folder list and its controls.
   In a root with many folders, select the last folder.
5. Press Esc. Press Tab to open Audio. Check the click level.

### SMC-PAD

1. Open Setup. Release PAD BANK. Select Audio with LEFT or RIGHT.
2. Select Click level with E1. Note the percent.
3. Press LEFT to open Paths. Check the selected root.
4. Turn E2. Check that it stays dim. Hold PAD BANK and press P5 Choose.
   On 80x26 and 120x40, turn E1 to the last folder. Check that its name and P1/P4 stay visible.
5. Hold PAD BANK and press P1 to cancel. Release PAD BANK. Press RIGHT for Audio. Check the click level.
6. Press RIGHT to open Input. Turn E1 and E2. Check that both stay dim. Check the input log.

### Sound

This change needs no new WAV. Check the saved Click level in Audio.
