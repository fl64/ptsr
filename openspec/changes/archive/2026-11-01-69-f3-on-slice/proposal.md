# 69 — F3 on a slice pad opens on that slice

## Why

A UX check of the owner's agent (2026-11-01): F3 on a pad that plays a slice opens the
chop of the whole file, and the owner has to find the slice again. A chop belongs to the
file (42, 45), and that stays. "Open on your slice" (37) already opens on the slice for a
pad that follows the cuts of the chop (a pad that kept its slice after a re-chop opens the
older chop on it, 45). A pad that plays its own piece (its start or end moved by hand on
the SOURCE page, or its source set to the pad) opens with the view left as it was, often
the whole file.
The owner: "ok, do it".

## What changes

- F3 on any pad that plays a part of the file opens the chop with the slice where the
  piece starts selected and the view on the piece: the piece in the middle, half of its
  length on each side.
- The chop model stays: one chop per file, the pad's piece is not cut into it when the
  chop already exists.
