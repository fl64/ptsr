# 55 — The pad shows what is set on it

## Why

The owner (2026-11-01): "the main thing is not to lose what was set up: I turned on the
filter on a pad, how do I see it without opening the edit and diving into that block?
maybe show it on the pad, an icon?" Now the pad cell shows only the mode and the volume;
the panel shows every value of the selected pad at the same brightness, a changed one does
not stand out.

## What changes

- The pad cell carries a glyph on its top frame for every changed block: `↕` pitch, `◣`
  the envelope, `∿` the filter, `◻` a part of the file (a slice or an own piece, not the whole file). An unchanged block gives no glyph;
  a pad with nothing changed has a clean frame.
- The pad panel draws a changed value bright and a value at its default dim; the line of
  a block with a change carries `●`.
- F1 explains the glyphs.

The volume keeps its place on the mode line (`one-shot +3`), it takes no glyph.
