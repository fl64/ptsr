# Design 62

## Decisions (owner, 2026-11-01)

- "Yes" to the line below (shown to the owner):

```
now:      bank 1 ⢀⠂ · pad W kick
then:     bank [1 ⣿⣿] 2 ⠑  3 ⣿  4 ⠂  5    6 ⠿  7    8 ⠁   · pad W kick
```

- The same bank row the pads screen draws above the grid, so one look means the same on
  both. It counts as many banks as the program has.

- The flash in the bank row: a braille glyph holds 8 pads in one cell and one colour. Two
  takes did not read: only the hit dots in one half and the fill in the other ("part of
  the assigned pads disappears, the drawing is wrong"), then a white half with its dots.
  The owner: "show only what is held at the moment, so there is no unclear mess". While
  pads of the open bank are held or flash, both of its glyphs show only those pads, in
  white; the fill comes back after.
- The same bank row above the grid flashes the same way: one drawing for both views.

## Archive order

"Hide the pads" stands on 32: 62 is archived after 32.

## Open questions

None yet.
