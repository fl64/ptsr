# Proposal

## Why

The program ended with exit code 137 during fast pad play. The diagnostic mode of
change 128 showed the cause. The audio callback time grew from about 9 ms to 946 ms
in each second. The kernel then killed the real-time thread. An offline probe with
two reverbs repeated the growth after the pad hits ended: from 2 ms to 173 ms in each
second. With subnormal numbers flushed to zero, the time stayed at 2 ms.

Decaying reverb and filter tails make subnormal floating-point numbers. The processor
calculates them very slowly.

## What Changes

- At the start of each callback, the audio thread flushes subnormal numbers to zero.
- The callback benchmark adds a workload of pad hits followed by decaying reverb tails.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `audio-engine`: flush subnormal numbers in the audio callback.

## Impact

This change affects the audio callback and the benchmark. The sound changes only
below about -300 dBFS. The project and config formats stay the same.

## How to check

### Keyboard

1. Start the program. Hit pads with sends to two reverbs, fast, for one minute.
2. Check that the program keeps running.

### Controller

1. Start `target/release/ptsr --diag`. Hit SMC-PAD pads fast for one minute.
2. Read the `diag:` lines in the log. Check that `callback busy` stays below 50 ms/s.

### Sound

Listen to `renders/129-reverb-tails.wav`. The tails decay as before and end in silence.
