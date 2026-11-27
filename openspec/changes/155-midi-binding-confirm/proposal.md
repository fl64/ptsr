## Why

P4 can save a binding or answer a binding question. A pad or bank run also saves each control as soon as it receives it.

## What Changes

- Require a key to save, clear or reset a MIDI binding.
- Let P1 cancel. Show Enter for the step that saves the binding.
- Before each pad or bank in a series, show the selected control and wait for Enter.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- command-panel: reserve P4 for questions that allow controller confirmation.
- midi-learn: wait for a key before writing the map.
- controller-reach: show the keyboard step for binding changes.

## How to check

### Keyboard

1. Open F10 or select a row in F12 MIDI. Touch a controller control. Press Enter and reopen the map.
2. In a pad or bank series, press Enter after each control. Press Esc with a control pending. Check the saved rows.

### SMC-PAD

1. While a binding or reset waits for Enter, press P4. Check that the map stays the same.
2. Press P1. Check that the pending change closes. Repeat after a device reconnect.

### Sound

This change needs no new WAV. Save a pad binding, then hit that pad with the bound control.
