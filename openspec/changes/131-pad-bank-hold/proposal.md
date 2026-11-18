# Proposal

## Why

PAD BANK switches the pad layer inside the SMC-PAD and keeps it after release.
The program sees the button only through a status poll every 52 ms and switches
the panel on each detected press. A missed press or a layer left from an earlier
run puts the panel and the device layer out of step. On 2026-11-18 the panel
showed while the device sent first-layer notes, so a panel hit played a pad and
closed the panel. A measurement on the same day showed that the status reads
pressed for the full time that the owner holds the button.

## What Changes

- The panel shows only while the owner holds PAD BANK. Release hides it.
- Each hold shows the actions of the current screen or mode.
- With the hold shortcut on, a pad hit acts by the hold, not by the device layer.
  Held: the hit runs the cell at that position. Not held: the hit plays the pad.
- A visible question takes the No and Yes pads without a hold. The other pads play.
- The F12 MIDI setting `SMC-PAD PAD BANK` offers `Hold panel` or `Off`.
- Without the shortcut, the current layer rule stays: a second-layer hit shows the panel.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `controller-reach`: PAD BANK hold, status delays and the saved preference.
- `command-panel`: the second-layer rule without the shortcut and question pads.
- `input-devices`: the two note ranges of one physical pad.
- `screen-help`: the PAD BANK rule in F1.

## Impact

The change affects SMC-PAD status handling, pad note routing, F12 MIDI, F1 help and
the route table. The config key `midi_gestures.smc_pad_bank_opens_panel` keeps its
name and values, so no migration is necessary. Sound stays unchanged. No new
dependency.

Owner checks, controller: hold PAD BANK and check that the panel shows. Hit a cell,
release and check that the panel hides. Hit pads without PAD BANK on both device
layers and check that they play. Open Exit and answer with P11 or P12 without PAD BANK.
Owner checks, keyboard: keyboard keys and pad keys stay unchanged.
Renders: none, because sound stays unchanged.
