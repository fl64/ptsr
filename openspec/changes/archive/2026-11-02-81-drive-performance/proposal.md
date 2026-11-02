## Why

At 32 voices the drive takes much of the audio buffer time on a slow CPU core. The
benchmark also lets pitched voices end before the measurement is complete.

## What Changes

- Reduce repeated drive calculations without changing the drive sound or its smooth changes.
- Keep all 32 pitched voices playing throughout the benchmark.
- Compare the output and CPU budget before and after, including a moving drive control.

## Capabilities

### Modified Capabilities

- `pad-playback`: preserve the drive sound and smooth parameter changes under optimization.

## Impact

Audio engine, offline tests and benchmark. No new dependency or project format change.
Scheduler delays and audio backend configuration are out of scope.

## How to verify

Listen to the before-and-after drive render. The two halves must sound the same, including
while the drive changes. Run the benchmark on the same CPU core before and after.
