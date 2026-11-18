# Proposal

## Why

The program sometimes ends with exit code 137. The kernel or another process sends
SIGKILL, so the program cannot write a message. The log shows no error before the end.
The owner wants a diagnostic mode that can stay in the program for later problems.

## What Changes

- Add a diagnostic mode. Start the program with `--diag` or with `PTSR_DIAG=1`.
- In diagnostic mode, write each keyboard action and each MIDI input to the log.
- In diagnostic mode, write a log line when one audio callback runs longer than 100 ms.
  The line gives the duration and the callback stage.
- In diagnostic mode, log the callback count, the callback time and the CPU time of
  each real-time thread once a second.
- In diagnostic mode, lower the soft real-time CPU limit below the hard limit.
  The kernel then sends SIGXCPU first, and the program writes it to the log.
- Without diagnostic mode, the log stays as it is now.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `audio-engine`: record the callback start and stage for a diagnostic watch.

## Impact

This change affects the audio callback, the log and the program start. The callback
writes only atomics. The project and config formats stay the same.

## How to check

### Keyboard

1. Start `target/release/ptsr --diag`. Press some keys.
2. Open `~/.local/state/ptsr/ptsr.log`. Check one line for each action.
3. Start the program without `--diag`. Check that the log has no action lines.

### Controller

1. Start the program with `--diag`. Hit pads, turn knobs and press buttons.
2. Check one log line for each MIDI input.
3. Repeat the steps before exit code 137. After the exit, read the last log lines.

### Sound

The sound stays the same. No render is necessary.
