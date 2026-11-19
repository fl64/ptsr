# Design

## Owner decisions, 2026-11-19

- Chop is an action on a pad (option Q3.1). The Chop screen stays in the screens view
  for a return to the same chop.
- The slice page shows where the owner works, not the last pad bank.

## Decisions

### D1. The chop cell

The Pads actions get `chop` in cell 11 (device pad 11). It runs the same route as F3 on
the selected pad. On an empty pad it shows a notice and stays on Pads.

### D2. The first slice page

When Chop opens from a pad, the slice page is the page of the slice that the pad plays.
For a whole file, it is the first page. LEFT and RIGHT keep their page roles.

### D3. One Undo for an apply

An Assign apply writes one history step with every target pad before the apply. One
Undo restores all of them: old sounds come back and empty targets become empty again.
The chop stays open with its cuts.

## Corner cases

| # | Case | Expected result | Status |
| --- | --- | --- | --- |
| E1 | Apply over pads on two banks | One Undo restores both banks | test planned |
| E2 | Undo after Apply, then Redo is dim | Redo stays unavailable, as now | test planned |
| E3 | Chop from a pad that plays slice 20 of 40 | Page 17–32 shows, slice 20 selected | test planned |
| E4 | Chop from an empty pad | A notice, Pads stays | test planned |

## Open questions

None.
