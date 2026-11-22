## Why

A held pad, a preview, or an effect tail can keep sounding after STOP. The owner needs one fast way to silence the whole sampler and retain the current take.

## What Changes

- Add a Panic action on Ctrl+Period. On SMC-PAD, press STOP twice while its command panel is open.
- Show the controller gesture and its second press. Keep the normal STOP path outside that gesture.
- Stop the clock, cancel count-in, and fade pads, preview, click, and shared effects. Keep the accepted part of a pattern or audio take.
- Let the next pad or PLAY input start sound again.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- audio-engine: cut all playback with the short fade and clear old effect tails.
- keyboard: bind the reserved Panic chord.
- controller-reach: use two SMC-PAD STOP presses with the panel open.
- screen-help: show the gesture, armed state and result.

## Impact

The command uses the existing audio queues and the existing short fade.

## How to check

### Keyboard

1. Play a pad with a long tail and start A1. Press Ctrl+Period.
2. Check that the sound fades and the clock stops. Hit a pad again.
3. Repeat during a live A1 pass. Check that the recorded hits play after restart.

### SMC-PAD

1. Play A1 and hold PAD BANK. Press STOP once. Read the second-press hint.
2. Release PAD BANK to cancel. Check that A1 keeps playing.
3. Hold PAD BANK again. Press STOP twice. Check the result. Release PAD BANK and play again.

### Sound

Listen to renders/141-panic.wav. The old sound goes silent after the short fade. The next hit plays.
