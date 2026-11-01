# 45 — A file keeps every chop that has pads

## Why

The owner (2026-10-31): "the edit seems broken: F3 on a pad with a slice shows the
unmarked source sample; maybe it broke when I chopped the same sample for other pads".
A file had one chop: F3 on the whole-file pad started a fresh one (42) and the cuts the
older pads came from were gone, so F3 on such a pad opened the empty fresh chop. Picked
"Q1.1": a file keeps several chops, each with its pads.

## What changes

- A fresh chop of a file (F3 on the whole-file pad) does not take the older chop away: the
  older chop stays with its pads, they keep saying `slice N` and follow its cuts.
- F3 on a pad opens the chop its slice came from, with all its cuts.
- The files screen and F3 on the whole-file pad work on the newest chop, as now.
- Projects saved before keep working: a chop that is gone but still has pads is rebuilt
  from their bounds when the project opens.
- The project file gets the older chops of a file; absent reads none, older projects
  open unchanged (the migration is the rebuild above).

## Not in this change

`Y` and `Ctrl+Y` on a laid-out chop still replace its cuts and let its pads go, with undo,
as now (27, 35).
