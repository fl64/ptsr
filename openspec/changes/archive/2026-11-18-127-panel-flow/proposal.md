# Proposal

## Why

The owner gets lost in the SMC-PAD command panel. The main page mixes screen moves
and screen actions. Global actions repeat on the context pages of every screen.
More, Next and Main change the meaning of the same pads. The cell numbers on the
screen count from the top, but the device prints its numbers from the bottom.

## What Changes

- PAD BANK opens the actions of the current screen.
- While the panel shows, LEFT shows the screens view and RIGHT shows the actions.
- A screen or mode has at most 12 actions on one page. An action that does not fit goes
  to its own screen or mode.
- The screens view opens a screen with one hit. It also has Help, save, open,
  learn and exit.
- Every view keeps back, undo, redo and panic on the bottom row.
- The panel draws each view in its own color. A header row names the view and the
  views on the left and right. The cells show the numbers printed on the device.
- The target view keeps undo on its fixed cell. Apply moves to the panic cell.
- LEFT and RIGHT on Chop select pad banks, as on Pads.
- The footer shows only bank 1 knobs in physical order and a short button summary.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `command-panel`: views, cell layout, target cells and the panel file format.
- `controller-reach`: LEFT and RIGHT on a visible panel, Help route and press counts.
- `screen-layout`: panel colors, device numbers and the footer.

## Impact

This change affects the SMC-PAD panel, its file `config/panel.toml`, the footer
and the route table. No owner copy of the panel file exists, so the new format needs
no migration. The keyboard routes stay the same. The sound stays the same.

## How to check

### Keyboard

1. Use the current keys on Pads, Chop and Files. Check that they work as before.

### Controller

1. On Chop, press PAD BANK. Check that the panel shows the Chop actions.
2. Press LEFT. Check the screens view. Hit FILES. Check that Files opens with its actions.
3. Press LEFT. Check the screens view. Press RIGHT. Check the actions again.
4. Read the header. Check the view name and the views on the left and right.
5. Check that the cell numbers match the numbers printed on the device.
6. Start Copy. Check undo on its cell and apply on the panic cell for Assign.
7. Read the footer. Check the knob grid and the button summary.

### Sound

The sound stays the same. No render is necessary.
