# 37 — The chop screen says where you are, by the picture

## Why

The owner (2026-10-31): "I open the pad of a given slice in the editor, I see all the
slices, and which was the source pad I have no idea — I am lost". A text in the header
was refused: "text is a thing you have to find and understand first". The picture must
make it obvious.

## What changes

- **F3 from a pad that plays a slice opens on that slice**: the view shows the slice in
  the middle with half of it on each side, the neighbours at the edges dim; `K` zooms out
  to the whole file. From a pad that plays the whole file the view stays as it was.
- **The source is marked**: the bank row marks with `◆` the banks that hold a pad playing
  the whole file; on the grid that pad says `◆ all` and is not dimmed.

## How to verify

F3 on pad 2-Q (slice 5): slice 5 fills the middle of the wave, the grid has Q selected,
the bank row shows `1◆`; `Ctrl+1`: the source pad says `◆ all`; `K`: the whole file.
