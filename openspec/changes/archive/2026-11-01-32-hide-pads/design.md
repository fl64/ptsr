# Design 32

## Model

A view switch, not a screen: the frame of 31 gets a second shape. Shown, as in 31.
Hidden: the strip stands in the place of the grid, the left column from the title line
down, the pad line under it; the right column keeps the full height.

## Decisions (with the owner, 2026-10-31)
- Hidden, the strip grows in the left column only (the owner took the default, 2026-10-31):
  the first try gave the strip the full width and all the rows, and the right column was
  left one row high — the meters showed only R, the files list one line. The rows the
  grid frees are on the left; the right column still needs its own. The wave gets much
  taller and as wide as the grid (108 of 150 columns); for a fine cut `Ctrl+O` brings the
  pads and the full width back.
- The help names the state among the Ctrl keys, shown while Ctrl is held, as the other
  Ctrl keys (21 "Help for a held modifier"). Without kitty the usual line of F3 would
  need a fourth row at 140 columns ("Help line fits the terminal"), so there only F1
  lists the switch.

- Hide instead of compact pads ("maybe just hide the pads").
- One line instead of the grid: the bank and the selected pad, flashing on a hit.
- The assign mode of F3 brings the grid back while it is open.
- `Ctrl+O`: Ctrl under the left thumb, O under the right hand; `Ctrl+U` was the other
  free letter. `Ctrl+H` and `Ctrl+M` are taken by Backspace and Enter without kitty.
- Kept across restarts in `[ui]` as `hide_pads`, default `false`: an old config without
  the field reads as "shown", no migration.
- The switch is global: one state for F2, F6 and F3.

## Open questions

- The bank glyphs on the pad line: only the open bank keeps its glyphs (`bank 1 ⢀⠂ · pad W
  kick`). All eight banks take about 53 of the 60 columns of the grid at 100 columns and
  cut the pad name; the bank row of the shown grid keeps all eight.
