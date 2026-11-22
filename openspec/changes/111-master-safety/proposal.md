## Why

Several loud pads and shared effects can overload the master. The owner needs a safe output and a way to balance both FX returns.

## What Changes

- Limit the master with a minus 0.3 dBFS sample ceiling. Use 1.5 ms of look-ahead.
- Keep the cue after the limiter and align it with the pads.
- Preserve the pattern grid, recorded ticks and five-millisecond Panic deadline.
- Put an independent return level and mute on each slot of the FX screen.
- Show the active limiter when gain reduction exceeds 1 dB.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- audio-engine: limiting, output latency and Panic.
- master-volume: reference level and gain reduction feedback.
- transport: keep the cue in time with the limited output.
- patterns: keep the grid after the fixed output delay.
- pad-playback: keep Pitch Gate at the grid boundary.
- projects: save the return controls of both shared FX slots.
- screen-help: FX controls and the limiter mark.

## How to check

### Keyboard

1. Load a loud set of pads. Play a dense passage with both FX slots.
2. Check the limiter mark. Listen for distortion or pumping.
3. Open FX. Select one slot. Change its Return level with Left or Right.
4. Press M to mute its return. Select the other slot and compare.

### SMC-PAD

1. Open FX from the screens panel. Select a slot.
2. Turn E5 to change Return. Open the panel and press P13 to mute it.
3. Release PAD BANK. Check the level and mute state of each slot.

### Sound

Compare the normal and loud master renders. Check the click against the pad attack.
Listen to the return and mute changes while a delay or reverb tail sounds.
