# Design

## Owner decisions, 2026-11-20

- The zoom works around the point that the owner touched last: the cursor, a slice bound or a region edge.
- All four bounds count: the slice start and end, and the region start and end (option Q1.1).
- The cursor does not move to a touched bound. After a zoom around a bound, the cursor can be
  out of the view. The next cursor move brings the view back (option Q2.1).
- This is change 136. Its code comes after change 132 merges and before change 135 (option Q3.1).

## Decisions

### D1. The anchor

Each chop keeps one anchor. It starts at the cursor. A move of the slice start, the slice end,
the region start or the region end makes that point the anchor. The anchor follows the point,
also when an undo or a later move changes its position. A cursor move, a pad hit, a slice
selection and a new chop make the cursor the anchor again. The anchor is not saved.

### D2. A zoom around the anchor

Each zoom control keeps the anchor at the same place of the picture. Now the zoom does this with the cursor.
When the anchor is a bound, the zoom does not pull the view to the cursor.

### D3. The names of the old requirements

The requirement names "Zoom around the cursor" and "The cursor and the zoom on the right hand"
stay, so the archive keeps its history. Their text names the anchor.

## Corner cases

| # | Case | Expected result | Status |
| --- | --- | --- | --- |
| E1 | Move the slice start, then select another slice with a pad hit | The cursor is the anchor again | test |
| E2 | Move the slice end, then undo the move | The anchor stays on the slice end at its old position | test |
| E3 | Zoom in around the slice end, the cursor leaves the view, then move the cursor | The view comes back to the cursor | test |
| E4 | Move the region start, then zoom out to the whole file | The whole file shows, as now | test |
| E5 | Delete the cut that was the anchor | The cursor is the anchor again | test |
| E6 | Assign or a question is open | Zoom stays blocked, as now | keep |

## Open questions

- A bound that walks off the edge of the view: the view does not follow it in this change. Default: keep as now.
