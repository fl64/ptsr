# 04e — Chop screen polish

## Why

The agent went through the whole chop path on a 140-column terminal (without sound). It works, but in several places the screen hides or blurs what the owner needs at that moment.

## What changes

- The help line at the bottom wraps onto a second line when it does not fit; the key of the main step of the screen comes first (Enter on the chop screen).
- The plan summary next to the grid wraps instead of being cut.
- After a restart Tab goes on from the current number of slices (8 → 16), not back to 2.
- In the listen mode the pad of the selected slice is highlighted on the grid.
- The overwrite question names each pad plainly: "bank 1, pad 3: kick → slice 3".
- Empty pads in the assign mode show only "empty", without "one-shot".
- With one slice the side panel says "pad 1 plays the whole sample".

## Out of scope

The height of the waveform; kitty graphics (04b).

## How to verify

1. Terminal 140 columns wide, F3 on a sample: the help line shows `[Enter] to pads`; on F2 it shows `[F12] settings`.
2. Tab to 8, quit, start again, F3, Tab: 16 slices.
3. Hit pad W in the listen mode: pad W is highlighted.
4. Enter, hit a pad that holds a sample, Enter: the question says "bank 1, pad 3: kick → slice 3".
5. The summary on the right is read in full; empty pads say only "empty".
