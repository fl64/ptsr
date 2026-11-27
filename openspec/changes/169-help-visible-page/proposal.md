## Why

With SMC-PAD connected, the MIDI footer can take one more row. Help draws a shorter page, but a page press still moves by the full terminal height. It skips one row.

## What Changes

- Use the height left after the MIDI footer for Help navigation.
- Use that height for Messages and the Controller list too.
- Keep the current tab, device, and scroll rules.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- screen-help: advance by the number of rows on the screen.

## How to check

1. Open F1 on Pads with an SMC-PAD connected at 80x26.
2. Press Alt+PageDown and read the next range. Press P5 to go back. Press P6 to go forward.
3. Repeat at 120x40. Switch to Messages, page in each direction, and resize.
4. Disconnect the controller with Help open. Page again with the keyboard.

## Sound

This change needs no WAV. It changes Help navigation.
