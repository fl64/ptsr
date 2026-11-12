# Proposal

## Why

The owner opened the pad edit on an empty pad. The strip showed the live output, not the pad's sound with its envelope. An empty pad plays its test tone. It is a 1 s decaying tone with its own note. Volume, pitch and the envelope act on that tone. The strip did not show it. "Pad edit strip replaces live output" already requires the pad view in the edit.

## What Changes

- In the pad edit of an empty pad, the strip shows the pad's test tone as its piece. The envelope line stands over it, as for a sample.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pad-playback`: the pad edit of an empty pad shows its test tone.

## Impact

The pad edit view only. No sound, input or saved-format change.

## Out of scope

None.

## How to check

1. Select an empty pad on F2 and press Enter.
2. Check that the strip shows a decaying wave with the envelope line, not the live output.
3. Raise the attack: the line rises over a longer part of the wave.
