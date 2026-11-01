# Design 29

## Layout

- A pad cell grows from 4 rows (frame, name, mode, frame) by `w` wave rows under the mode
  line, `w` from 0 to 3: the most that fits the body of the pads screen,
  `w = min(3, (rows for the grid − 16) / 4)`. The grid takes `16 + 4w` rows: 16, 20, 24
  or 28. With `w = 0` the cells are exactly today's (the 26-row test keeps passing).
  (Owner, 2026-10-30: "3 rows, or adapt to the screen".)
- The info column keeps the panel; the area under it is left empty (the big view of 04b
  is not drawn).

## The wave of a pad

- What the pad plays: its region of its sample, peaks per Braille dot column (two per
  cell), multiplied by the pad's envelope over the piece (attack from 0 at the start,
  decay to 0 at the end; the release is not drawn, it acts after a gate release).
- `w` rows of Braille, mirrored around the middle like the strip; the colour is the dim
  wave colour, the selected pad's wave in the pad's text colour.
- Cost: the peaks are kept per pad and computed again only when its sound, region or
  envelope changes; a frame never scans samples.
- An empty pad: a flat middle line; a missing file: no wave.

## Open questions

- The wave colour per bank (as the frames) — default: no, dim.
