# Proposal

## Why

On 2026-11-20 the owner saw the hints change between SMC-PAD words and keyboard words at each
pad hit. The SMC-PAD sends each hit on two ports: Master and Private. Private repeats Master.
The program used SMC-PAD words only when the last hit came from Master. When the Private copy
came last, the program showed keyboard words.

## What Changes

- A message on the Private port of the SMC-PAD does not change the last input device. The hints keep SMC-PAD words.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `screen-help`: the two ports of one SMC-PAD count as one input device.

## Impact

The change affects the choice of hint words and the input indicator only. Keys, profiles,
saved files and sound stay unchanged. No new dependency.

Owner checks, controller:

1. `PADS` → hit pads 1–16 several times → the hints at the bottom keep SMC-PAD words and do not blink.
2. `PADS` → press a key on the keyboard → the hints change to keyboard words → hit a pad → SMC-PAD words come back.

Renders: none, because sound stays unchanged.
