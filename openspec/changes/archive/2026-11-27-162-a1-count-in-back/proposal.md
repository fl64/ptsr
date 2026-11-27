## Why

During the A1 count-in, Esc and SMC-PAD P1 leave the overview. The count-in continues on Pads.

## What Changes

- Cancel count-in with Esc or SMC-PAD command pad P1. Keep the A1 overview open.
- Show the current cancel control beside the step numbers.
- With Hold panel off or unavailable, accept command P1 while the panel is hidden.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- patterns: keep the overview open after the owner cancels count-in.
- command-panel: accept P1 for count-in before panel reveal.
- screen-help: show the available cancel control.

## How to check

### Keyboard

1. Press F5. Stop playback if it runs.
2. Press Ctrl+R, then Esc during count-in.
3. Check that count-in stops and the A1 overview stays open.

### SMC-PAD

1. Open A1. Stop playback if it runs.
2. Tap RECORD. During count-in, hold PAD BANK and press P1.
3. Check that count-in stops and the A1 overview stays open.
4. Repeat with Hold panel off. Use P1 in the second bank.

### Sound

The existing cancel command stops the count-in clicks. This change needs no new WAV.
