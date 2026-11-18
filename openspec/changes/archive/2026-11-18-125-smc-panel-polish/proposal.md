# Proposal

## Why

The SMC-PAD layout gives the knobs direct routes. The panel also needs clear
labels and a short path from a command to its target pad.

## What Changes

- Make PAD BANK open the panel by default after the supported USB check.
- Add Mixer, Settings and More to the second main row. Move Help into More.
- Show pad targets on the active panel for Copy, Take, a file in hand and Assign.
- Keep confirmation cells visible for replacements. Show a route to pads 13–16.
- Warn about shifted pads and an unsupported active preset.
- Explain device Note Repeat in F1 and the setup guide.
- Show full knob operations in Controller Help. Add Tap Tempo on Pattern More.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `command-panel`: main cells, names, target cells and safety controls.
- `controller-reach`: PAD BANK default and shorter target routes.
- `input-devices`: warn about shifted pad notes and the active preset.
- `screen-layout`: show the selected knob operation.
- `transport`: set Pattern tempo from taps.

## Impact

This change affects the SMC-PAD command panel, USB status handling and help.
It builds on the knob layout in change 124. Both changes share one owner test.

## How to check

### Keyboard

1. Use F1 from Pads. Close Help with Esc.
2. Copy a pad with Ctrl+C. Hit an occupied target. Cancel with Esc.
3. Try again. Accept the same target with Enter. Check the source and target.

### Controller

1. Connect the supported USB unit. Open the panel with PAD BANK.
2. Press More. Open Help. Close it with Back.
3. Start Copy, Take or Assign. Select a target on the active panel.
4. Try a target on the bottom row of the playing layer. Return to Apply.
5. Cancel and accept a replacement on separate runs.
6. In Pattern, press More and tap P6 in time. Check the BPM.
7. Check both terminal sizes. Compare the visible counts with the route table.
   Use `pads-panel-targets` and `chop-panel-targets`.

### Sound

Use `renders/92-command-panel.wav` as the transport reference. Listen to a held
Gate while the panel opens and closes. Listen to each target after placement.
