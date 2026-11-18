# Design

## Owner decisions

On 2026-11-18 the owner asked for one clear panel flow, like an MPC or SP-404:

- PAD BANK opens the actions of the current screen.
- LEFT and RIGHT on the visible panel move between the screens view and the
  action pages. Lists do not move while the panel shows.
- The bottom row always has back, undo, redo and panic.
- Each view has its own color. The cells show the device numbers.
- The owner will test the build and ask for changes.

## Views

The panel has a row of views: `screens`, `actions 1`, `actions 2` and later pages.
A reveal by PAD BANK or a first second-bank hit shows `actions 1`. LEFT moves one
view to the left. RIGHT moves one view to the right. The row does not wrap.
A screen change shows `actions 1` of the new screen. The target view and a visible
question keep their own cells, and LEFT and RIGHT do nothing there.

This is like the MPC menu: one level selects the mode, the next level does work in it.

## Cell numbers

The specs and the panel file count cells from the top left: cell 1 is the top left,
cell 16 is the bottom right. The device prints its pad numbers from the bottom left.
The panel draws the device number: `(3 - row) * 4 + column + 1`, with row 0 at the top.
Questions keep cells 7 and 8. Their device numbers are 11 and 12.

## Screens view

| Row | Cells |
| --- | --- |
| 1 | pads · files · chop · A1 |
| 2 | fx · mixer (dim) · setup · help |
| 3 | save · open · learn · exit |
| 4 | back · undo · redo · panic |

A screen cell opens that screen and shows its `actions 1`. Help opens F1 Controller.
Exit asks the existing question.

## Action pages

Each page has up to 12 cells in rows 1 to 3. The frequent actions are on page 1.
A global action is in the screens view only. `play` duplicates the PLAY button and is
gone. A dash is an empty, dim cell.

| View | Page 1 | Page 2 |
| --- | --- | --- |
| Pads | edit · copy · take · mark / mode · clear · gate · marks / grid · output · — · — | grid - · grid + · rows · step / A1 rec · erase · A1undo · knobs |
| Pad edit | mode · clear · gate · copy | — |
| A1 | step · grid - · grid + · knobs / record · erase · undo · clear / rows · tap · — · — | — |
| Files | take · load · parent · star / enter · auto · chop · — | — |
| Chop | cut · parts · hits · assign / loop · play · snap · as pad / start · end · whole · commit | hits - · cut - · end - · order / level · edit · — · — |
| Chop edit | level · hits - · cut - · end - / order · — · — · — | — |
| FX | — | — |
| Settings | tab · learn · clear · — / row - · row + · — · — | — |

## Target view

Cells 1 to 12 stay target pads of the current bank. The bottom row is
back (cancel) · undo · menu · apply. Undo keeps its normal role. Apply is active only
for Assign. This fixes the old trap where cell 14 changed from undo to apply.

## Look

Action cells use cyan, the screens view uses magenta, the bottom row uses grey text.
The header row reads, for example, `‹ screens │ CHOP actions 1/2 │ actions 2 ›`.
A cell shows its device number and label. Empty cells show only the number, dim.

## Footer

The footer shows bank 1 as a 2 by 4 grid in physical order, without bank 2.
Bank 2 has only master, so one line says `B2 8 master`. The marked knob and the value
line from change 126 stay. The right side shows the buttons: LEFT and RIGHT with
their current role, PAD BANK, PLAY and STOP. The keyboard hints follow while rows remain.

## Open questions

- The target view menu cell (cell 15) is kept. The default keeps its current role.
- Mixer stays dim until the mixer screen exists.
