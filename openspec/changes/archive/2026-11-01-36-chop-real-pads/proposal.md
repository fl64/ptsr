# 36 — The chop screen shows the real pads

## Why

The owner (2026-10-31) loaded a long sample on a pad, chopped it, laid the slices out and
opened the chop screen from the source pad again: the grid showed "slice 1…16" on the pad
keys, while the slices lay on other pads — "is this clear at all? on which pads? is it
logical?". The grid of the listen mode was a keyboard of slices that looked like the pads.
And a cut inside a laid-out slice unlinked its pad: an exception to how pads follow cuts.

## What changes (asked one by one, 2026-10-31)

1. The listen grid shows the real pads of the bank, as on F2; F3 opens on the bank that
   holds the slices; `Ctrl+1…8` turn banks; pages of slices go.
2. Places are written with a dash: `2-4`, `2-Q`, `on 2-1`.
3. The bank row is shown in the listen mode too, banks with slices of the file marked.
4. A cut inside a slice on a pad keeps the pad on the left part (the slice that still
   starts at its cut); the status names the pad that got shorter.
5. Slice numbers wear `●` in the bank color (on a pad) or a dim `○` (on no pad); a hint
   says how to place the `○` slices.
6. A new pad starts with the default settings (nothing is copied).

## How to verify

Recorded-event and drawn-screen tests per scenario; live: F3 from the whole break opens on
the bank of its slices, the grid plays the pads; a cut inside slice 5 shortens its pad,
the new slice shows `○`, Ctrl+Z returns.
