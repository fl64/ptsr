## Why

The MIDI footer can tell the owner to hold PAD BANK or use it with P1 when the shortcut is off or unavailable. With the shortcut off, the first hit in pad bank 2 shows the panel. The next P1 closes Pad Edit or Slice Edit.

## What Changes

- Name pad bank 2 in the footer when it opens the commands.
- In Pad Edit and Slice Edit, show two P1 presses when the panel is hidden and one when it is visible.
- Keep the hold instruction for a verified controller with the shortcut on.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- screen-help: name the current route to the panel and out of an editor.

## How to check

1. With Hold panel off, open Pad Edit or Slice Edit at 80x26 and 120x40.
2. Switch to pad bank 2. Press P1 once and then again. Read the hint after each press.
3. With the shortcut on and a verified controller, hold PAD BANK and press P1.

## Sound

This change needs no WAV. It changes screen instructions.
