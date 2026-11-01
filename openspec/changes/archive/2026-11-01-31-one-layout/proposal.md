# 31 — One layout for every screen

## Why

The owner (2026-10-31): the pads look different on the pads, files and chop screens, and
the files screen puts the folder path above the pads while its list is on the right. "I
just want some uniformity": the wave always above the pads, always one height, on the
pads screen, in the edit and everywhere; one layout, the modes differ.

## What changes

- **The wave strip on top**: every screen with pads (F2 pads, F6 files, F3 chop) draws
  the same strip above the grid, of the same height: all the rows the pads leave, at
  least 8, fewer only when the terminal is low. F2 shows the live output (the pad envelope in the edit, the recording while it
  runs), F6 the file under the selection, F3 the sample with its cuts and the cursor.
- **The same pad grid**: the same cell size and the same pad waves (29) on every screen.
- **The right column is the mode**: F2 the pad panel, F6 the folder path and the filter
  line right above the list, F3 the slice line and the side list.
- **What stays**: the status line on top, the help at the bottom, the keys, what each
  mode shows; only where it stands changes.

## Out of scope

Settings (F12) has no pads and keeps its layout; the compact pads (BACKLOG #15); kitty
pictures for the pad waves.

## How to verify

Switch F2 → F6 → F3 → F2: the strip and the pads stay in place, only the right column
changes. F6: the path stands right above the list. F3: cut, zoom, the cursor work as
before on the lower strip. Make the terminal lower step by step: the pad waves go first,
then the strip shrinks (down to 3 rows), the grid always fits. Tests: drawn screens of F2, F6, F3 at the
same terminal size have the strip and the grid at the same rows and columns.
