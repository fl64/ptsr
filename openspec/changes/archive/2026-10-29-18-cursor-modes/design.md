# Design — 18

## Hits

Computed once when the chop opens, from the left channel: blocks of 256 frames, the block peak. A hit starts in a block whose peak is at least 1.5× (+3.5 dB) the highest peak of the 2 blocks before it (~10 ms) and above −40 dBFS; hits closer than 50 ms to the previous one are dropped. The hit position is the first frame of that block whose level reaches half of the block peak. The list lives in `Chop` and is sorted; 04d replaces the detector with aubio onsets.

## Moves

- Even: `step = view / 32` (at least a frame), clamped into the region.
- Fine (Ctrl, both modes): `step = view / 256`, at least a frame.
- Hits: the next hit strictly after the cursor (before it for ←) inside the region and inside the view; none → an even step. The view follows the cursor as now, so the next hit comes into view and snapping goes on.

The mode is a `Chop` field, off when a chop opens (not saved in the session).

## Key repeat

`Keymap::resolve` lets repeat events through for `CursorMove`, `MarkerNudge` and `ChopZoom`; everything else, pads included, still ignores them.

## Open questions

- The detector thresholds were picked on one breakcore amen loop (5.6 s): twice over 40 ms found 11 hits, 1.5× over 10 ms found 39, most on an even ~176 ms grid. Not checked by ear; the owner listens.
- 04d plans `N` as "snap cuts to hits", on by default. Here `N` is one switch "snap to hits" for the cursor, off by default; 04d must extend the same switch to cuts and decide whether its default changes. Default taken: one switch, off.
