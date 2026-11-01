# 42 — F3 from a whole-file pad starts a fresh chop

## Why

The owner (2026-10-31): "I open the same sample twice, put it on pads, and if it was
chopped before, even the new one shows chopped — a bug?". The cuts belong to the file
(27), so any pad of the file opened the old chop. Picked: "always a new one — I opened
it again for a reason, why would I want the old chop".

## What changes

- F3 from a pad that plays the whole file starts a fresh chop: no cuts, the whole file.
- The pads of the old cuts keep playing what they played and let go of them (`pad`), as
  a new equal-parts chop does; the status says so; `Ctrl+Z` brings the old chop back.
- The old chop is edited from a pad that plays one of its slices.

## How to verify

Lay a break out, put it whole on another pad, F3 on that pad: no cuts; the old pads play
as before and say `pad`; `Ctrl+Z`: the old cuts and links are back.
