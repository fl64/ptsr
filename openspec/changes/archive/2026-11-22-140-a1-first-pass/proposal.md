# Show the next step on A1

## Why

The A1 overview gives the record key in its empty title. The owner still needs to know when to hit pads, finish recording, and hear the result.

## What Changes

- Show one short instruction to the left of the step numbers on A1.
- Show the record, count-in, pad, finish, and playback steps in order.
- Use the current keyboard bindings or the buttons of the SMC-PAD.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `screen-help`: the next step for a live A1 pass.
- `screen-layout`: the empty title and the instruction beside the step numbers.

## Impact

The A1 overview reads the current recording and playback state to select the instruction. The existing controls, pattern data, and sound paths remain the same.

## How to check

### Keyboard

1. Open F5 in a new project. Check that the screen names Ctrl+R.
2. Press Ctrl+R. Check the count-in and its cancel instruction.
3. Wait for recording. Hit pads. Check that Ctrl+R finishes the pass.
4. Press Ctrl+R. Wait for the recorded loop. Check the play or stop instruction for the current state.

### SMC-PAD

1. Hold PAD BANK. Press LEFT, then P4 (A1). Release PAD BANK. Tap RECORD. Check its name in the next step.
2. Wait for recording. Hit pads. Tap RECORD to finish.
3. Press STOP to pause the loop. Press PLAY to continue. Check the words on A1 after each press.

### Views and sound

Compare `renders/screens/pads-pattern-overview-absent-80x26.ansi` and `renders/screens/pads-pattern-overview-absent-120x40.ansi`. Listen to `renders/97-pattern-record.wav` as the live recording reference. This change adds no sound path.
