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

The panel has two views: `screens` on the left and the actions of the current view on
the right. A reveal by PAD BANK or a first second-bank hit shows the actions. LEFT shows
the screens view. RIGHT shows the actions. A screen change shows the actions of the new
screen. The target view and a visible question keep their own cells, and LEFT and RIGHT
do nothing there.

This is like the MPC menu: one level selects the mode, the next level does work in it.

## Twelve actions rule

On 2026-11-18 the owner set a design rule: a screen or mode has at most 12 actions.
The bottom row of the 16 pads does not change. An action that does not fit goes to its
own screen or mode, as pad edit does. The rule makes the panel easy to design. The owner
may lift it later.
On 2026-11-18 the owner kept 12 actions and the fixed bottom row back · undo · redo ·
panic on every view. The bottom row must stay the same and easy to read on every screen. The panel loader rejects a second action page with a notice.

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

## Actions

Each view has one page of up to 12 cells in rows 1 to 3. A global action is in the
screens view only. `play` duplicates the PLAY button and is gone. The pattern actions
of the old Pads page 2 are on the A1 screen. `whole` and the bound removal actions are
in Chop edit, and Chop gives `edit` to enter it. A dash is an empty, dim cell.

| View | Actions |
| --- | --- |
| Pads | edit · copy · take · mark / mode · clear · gate · marks / grid · output · — · — |
| Pad edit | mode · clear · gate · copy |
| A1 | step · grid - · grid + · knobs / record · erase · undo · clear / rows · tap · — · — |
| Files | take · load · parent · star / enter · auto · chop · — |
| Chop | cut · parts · hits · assign / loop · listen · snap · as pad / start · end · edit · commit |
| Chop edit | level · hits - · cut - · end - / whole · — · — · — |
| FX | — |
| Settings | tab · learn · clear · — / row - · row + · — · — |

## Banks on Chop

The owner could not reach a long chop on later pad banks from the controller. LEFT and
RIGHT on Chop now select pad banks, as on Pads. A pad hit selects a slice, as on an MPC.
Knob 1 moves the cursor. The keyboard keeps the slice keys.

## Pad and A1 commands

Take, output, grid, A1 record, erase, A1 undo and knobs were on a shared page of every
screen. Take, output and grid move to the Pads actions. The pattern actions move to the
A1 actions. From another screen, the screens view reaches Pads or A1 with one more press.
The keyboard keeps them on every screen.

## Target view

Cells 1 to 12 stay target pads of the current bank. The bottom row is
back (cancel) · undo · menu · apply. Undo keeps its normal role. Apply is active only
for Assign. This fixes the old trap where cell 14 changed from undo to apply.

On 2026-11-18 the owner chose a route for the Assign order. During Assign, the menu
shows the Chop actions with `order` in cell 4, because `assign` is already active.
Order does not act in Chop edit, so Chop edit does not show it.

## Look

Action cells use cyan, the screens view uses magenta, the bottom row uses grey text.
The header row reads, for example, `‹ screens │ CHOP actions │ ·`.
A cell shows its device number and label. Empty cells show only the number, dim.

## Footer

The footer shows bank 1 as a 2 by 4 grid in physical order, without bank 2.
Bank 2 has only master, so one line says `B2 8 master`. The marked knob and the value
line from change 126 stay. The right side shows the buttons: LEFT and RIGHT with
their current role, PAD BANK, PLAY and STOP. The keyboard hints follow while rows remain.

## Open questions


- The target view keeps the menu cell (cell 15). The default keeps its current role.
- Mixer stays dim until the mixer screen exists.
