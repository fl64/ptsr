## Why

The owner needs to place a pad in stereo, tune the pad in cents, and shape how
its level changes after a hit.

## What Changes

- Set Pan and Fine tune for each pad. Send the stereo result to the shared effects.
- Choose a linear or exponential envelope and set a hold after attack.
- Set the velocity range in decibels. Keep the raw hit velocity in A1.
- Show Pan, Fine tune, Velocity and Curve on the SOUND page and Hold on ENV.
- Keep the previous sound of an older project, including pattern hits on empty pads.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- pad-playback: stereo position, cents and envelope response.
- projects: save the new pad settings and open older data.
- editing: preserve the settings through copy, swap, Undo and Redo.
- screen-help: show the SOUND page and its controls.
- controller-reach: provide the same visible paths on SMC-PAD.

## How to check

### Keyboard

1. Open a pad edit. Use Tab to select SOUND. Select Pan and change it.
2. Change Fine tune. Check the shown semitones and cents.
3. Set the envelope curve and Velocity. Open ENV and set Hold.
4. Save and reopen the project. Check the settings and play A1.

### SMC-PAD

1. Open a pad edit. Hold PAD BANK and choose P14 Sound page.
2. Release PAD BANK. Turn E7 for Pan and E5 for Fine tune.
3. Hold PAD BANK and turn E5. Check the one-cent step. Release PAD BANK.
4. Turn E3 for Velocity and E1 for Curve. Open ENV and set Hold.
5. Hold PAD BANK and press P1 to return to the pad grid.

### Sound

Listen to the pan sweep in renders/144-pan.wav, reference pitch and cent
limits in renders/144-fine-tune.wav, and the change of attack hold and decay
in renders/144-envelope.wav. Compare the three ranges in renders/144-velocity.wav
and listen to the old response in renders/144-legacy.wav.
