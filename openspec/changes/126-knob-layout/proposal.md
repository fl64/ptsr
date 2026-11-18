# Proposal

## Why

The owner tried the SMC-PAD knob layout from change 124. On Pads, a knob turn changed
a value of a page that the screen did not show. A pad value had different knobs in
the two banks. The knob order was the reverse of the row order on the screen.
A short turn moved a value through its full range. The value pop-up was far from
the knob strip, so the owner did not see what changed.

## What Changes

- Outside pad edit, bank 1 on Pads keeps only knob 1 (pad selection) and knob 8 (master).
- In pad edit, knob 1 selects the page. Knobs 7, 5 and 3 change the first, second
  and third value of the page. This order matches the rows on the screen.
- Bank 2 keeps only master on knob 8.
- Each knob step changes a value by one step. Fast turns do not accelerate.
  The Pattern tempo knob changes 1 BPM per step.
- The knob strip marks the knob that turned last and shows its value and target.
- The F12 MIDI layout review also offers the new roles for a map from change 124.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `controller-reach`: change the Pads and pad edit knob roles and remove bank 2 pad roles.
- `input-devices`: remove relative knob acceleration and update saved maps from change 124.
- `screen-layout`: show the last turned knob and its value in the knob strip.

## Impact

This change affects the SMC-PAD profile, MIDI step handling, the knob strip and
the saved map review. It uses existing sound controls and keeps the project format.
The panel cell numbers follow in a separate change.

## How to check

### Keyboard

1. On Pads, open pad edit. Change the page and values with the current keys.
2. Check that the keyboard steps did not change.

### Controller

1. On Pads, turn knobs 2–7 in both banks. Check that no pad value changes.
2. Turn bank 1 knob 1. Check that the selected pad changes without sound.
3. Open pad edit. Turn knob 1. Check that the page changes.
4. On TONE, turn knobs 7, 5 and 3. Check volume, pitch and mute group.
5. Turn knob 7 slowly and quickly. Check that each step changes volume by 1 dB.
6. Look at the knob strip after each turn. Check the marked knob, value and pad.
7. With a saved map from change 124, open F12 MIDI. Review and apply the update.
   Turn bank 2 knob 1. Check that no pad value changes.

### Sound

This change adds no sound algorithm. Listen to the volume and pitch steps of one pad.
No render is necessary.
