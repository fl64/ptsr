# Design

## Owner decisions, 2026-11-19

- Chop is an action on a pad (option Q3.1). The Chop screen stays in the screens view
  for a return to the same chop.
- The slice page shows where the owner works, not the last pad bank.

## Owner decisions, 2026-11-20

- Each chop keeps its own slice page for every entry, from a pad and from Files (option Q1.1).
  The page keys turn that page. Pads keeps its bank.
- The row over the slice keys names the page. Separate `page -` and `page +` cells are not
  added, because LEFT and RIGHT turn the page (change 134 deleted such duplicate cells).

## Decisions

### D1. The chop cell

The Pads actions get `chop` in cell 11. The device prints P7 on that pad. It runs the same
route as F3 on the selected pad. On an empty pad, a notice names the pad and Pads stays.

### D2. The slice page of a chop

The slice keys show while no slice of the chop lies on a pad and no pattern work routes
the pads. Each chop keeps its slice page. Chop sets the page to the page of the selected
slice when it opens. A whole file and a new chop select slice 1, so they open on slices 1–16.
PgUp, PgDn, Ctrl+1…8, LEFT and RIGHT turn the page while the slice keys show. They do not
change the pad bank. With a slice on a pad, Chop shows the real pads of the bank, as now.
The page is not saved with the project.

### D3. One Undo for an apply

An Assign apply writes one history step with every pad that it changes, named `slices`.
One Undo restores all of them: old sounds and links come back and empty targets become
empty again. The chop stays with its cuts.

### D4. The page in the row over the slice keys

The row says `slices 17–32 · page 2/3`. The last page shows only the slices that exist,
for example `slices 33–40 · page 3/3`.

## Corner cases

| # | Case | Expected result | Status |
| --- | --- | --- | --- |
| E1 | Apply over pads on two banks | One Undo restores both banks | test |
| E2 | Undo after Apply, then Redo | Redo stays dim, as now | keep |
| E3 | Chop from a pad that plays slice 20 of 40 | Chop shows the bank of that pad, slice 20 selected | keep, test |
| E4 | Chop from an empty pad | A notice names the pad, Pads stays | test |
| E5 | Whole-file pad on bank 3, chop of 40 slices | Slices 1–16, page 1/3 | test |
| E6 | Turn the slice page, then go back to Pads | Pads shows the bank it had before | test |
| E7 | Files opens a chop on page 3, then the owner deletes cuts | The page goes to the last page that has slices | test |
| E8 | Pattern work routes the pads on Chop | The page keys select real pad banks, as now | keep |
| E9 | Undo after Apply, then F3 on a restored slice pad of an older chop | The older chop opens, as now | keep |
| E10 | Two Applies to the same pads within the undo join time | They join into one step, as other steps of the same name do | keep |

## Open questions

None.
