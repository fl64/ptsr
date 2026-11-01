# Design 31

## Model

One frame for the screens with pads: the status line, the screen header, the strip, then
the grid on the left and the mode column on the right, the help at the bottom. A mode
fills the strip and the right column; the frame decides the sizes.

## Decisions (with the owner, 2026-10-31)

- The strip is at least 8 rows and takes all the height the pads with their waves leave,
  the same on every screen (the owner: "8 rows may be too few, there is plenty of height,
  let it be more").
- The pad waves (29) are drawn on every screen with pads. The strip keeps its 8 rows before
  the pad waves get any, so a 40-row terminal gives the pads two wave rows, not three as
  in 29; three come from 43 rows up.
- The folder path above its list (from 30) is done here, as part of the right column.
- The strip spans the full width on every screen, so the chop wave keeps its width; on
  F2 the level meters leave the strip's right end for the top of the right column, over
  the pad panel.
- The bank row stands right above the grid on every screen (F2 had it in the title line).
- The pads take the same width on every screen: the side column keeps 40 columns, as F6
  and F3 do today; on F2 the pads get a little narrower in a terminal below 200 columns.
- The chop marks (the slice numbers with the cursor `!`, the green play bar) are the top
  two rows of the F3 strip, the wave the rest of it.
- The chop wave gets lower than today (it takes all free height now); the cut precision
  follows the width, which does not change.

## Archive order

29 (pad waves, "of the pads screen") and 30 archive before 31; when 31 is archived, the
"Pad waves" requirement of 29 widens to every screen with pads — the "One layout"
requirement already says so.

## Open questions

- Rows in a low terminal: who gives way first. Default: the pad waves go first (3 → 0),
  then the strip shrinks from 8 down to 3 rows: the bank row over the grid takes one
  row, and a 26-row terminal (29, "Short terminal") must still fit the whole grid.
- The slice line of F3: default — it goes to the top of the right column, like the path
  on F6, wrapped with the side list instead of cut.
- The in-hand line and the errors of F6: default — in the title line, after `F6 FILES`.
