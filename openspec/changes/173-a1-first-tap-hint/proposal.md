## Why

The keyboard starts Tap Tempo on A1. Its first-tap cue needs a connected SMC-PAD to appear.

## What Changes

- Show the first keyboard tap in the notice row.
- Remove the cue when the tap window ends or another tap sets the BPM.
- Keep the bank row, step numbers and warnings visible.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- transport: show the first tap from both inputs.
- screen-layout: keep the controller cue in the mode line.
- screen-help: give the keyboard cue space in the notice row.

## How to check

1. Open A1 with F5. Press Alt+B once. Check for the cue.
2. Press Alt+B again within two seconds. Check for the BPM result.
3. Wait more than two seconds after a first tap. Check that the cue disappears.
4. Open Tap Tempo on SMC-PAD with A1 P8. Check the mode line after the first tap.

### Sound

The cue uses the current tap sequence. This change needs no new WAV.
