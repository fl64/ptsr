# 60 — Ctrl+X swaps the pads whole, with every setting

## Why

The owner (2026-11-01): "Ctrl+X must swap the whole thing, the move and the pad settings".
The swap moved the sample, the mode, the volume, the pitch and the envelope, but not the
filter: the screen showed one filter on a pad while the engine played the other. The
requirement still named only "the sample, mode and volume" — it was written before the
pitch, the envelope and the filter existed.

## What changes

- A swap moves everything a pad has: the sample and what part of it it plays, the mode,
  the volume, the pitch, the envelope, the filter, the drive and the sends (56).
- A test swaps two pads with every setting off its default and checks both sides.
