## Why

Tap tempo covers the bank row on A1. The overview gives the same playback cue after pause and after reset. A live pass does not say if A1 already had hits.

## What Changes

- Put the applied tap tempo in the notice row.
- Show the accepted pause position and distinguish it from a stop at the start.
- Show First pass for empty A1 and Overdub for a pass that starts with existing hits.
- Keep the position and grid settings visible at 80x26 and 120x40.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- screen-help: show the effect of tap tempo and the accepted A1 state.

## How to check

1. Open A1. Tap tempo twice. Check the bank row and the notice.
2. Play A1 and press STOP once. Check the pause position and the resume cue.
3. Press STOP again. Check the stopped state and the play cue.
4. Record into empty A1. Check First pass after the count-in.
5. Record again with hits in A1. Check Overdub.

### Sound

This change uses the same transport and capture commands. It needs no new WAV.
