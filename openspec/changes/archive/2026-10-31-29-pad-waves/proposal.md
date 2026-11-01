# 29 — Every pad draws its wave

## Why

The owner (2026-10-30, a screenshot of the pads screen in the edit): the piece of the
selected pad was drawn twice — on the strip above the grid and as the "big view" under
the panel, where the narrow and tall area turned it into a spindle. The owner: "make the
pads a bit bigger and draw on every pad its envelope two rows high".

## What changes

- **Every pad draws its wave**: up to three more rows in each pad cell, as many as the
  terminal has room for, with the wave of what the pad plays with its envelope applied
  (the attack rises, the decay falls to the end). An empty pad draws a flat line. (Owner:
  the wave with the envelope, not the bare piece; "maybe 3 rows, or adapt to the screen".)
- **The big view goes**: the area under the panel is no longer drawn (04b task 3 is
  superseded). The strip above the grid stays: the live output, in the edit the envelope
  of the selected pad (21).
- **Short terminals**: the waves take fewer rows; when even one row does not fit, the
  pads are drawn as today, without waves.

## Out of scope

The playhead inside a pad; kitty pictures for the pad waves (they stay Braille); the
pad size beyond the two rows.

## How to verify

A tall terminal: every pad with a sample shows a wave in three rows; make the terminal
lower step by step — the waves take two rows, then one; turn the attack of a
pad up — its wave rises slower at the start; an empty pad shows a line. Make the terminal
26 rows high: the pads are as before, without waves. Nothing is drawn under the panel.
