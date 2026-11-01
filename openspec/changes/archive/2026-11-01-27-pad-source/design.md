# Design 27

## Model

- The pad keeps one source: `All`, `Slice(id)` (the link to the cut at the slice's start,
  as the link of today), or `Own(start, end)`. The slice number shown is computed from
  the id, so a delete or a new cut renumbers it as the chop does.
- Today's pads map without migration: a linked pad is `Slice`, a pad whose region is the
  whole file is `All`, an unlinked pad with a region is `Own`. The session keeps the
  fields it has today; the source is derived on load.
- The SOURCE page: knob 1 steps the source over its list (all, slices 1…N, pad); knobs 2
  and 3 move the start and the end in the file. A move on `All` or `Slice` makes the pad
  `Own` with the current bounds first, one undo step.
- The chop screen lists the pads of each slice from the same links it uses to follow the
  cuts.

## Decisions (with the owner, 2026-10-30)

- The linked school, made visible as on an Akai MPC: "convenient; transparent and logical
  for someone coming from an Akai".
- The assign always links; a pad is unlinked one by one on its SOURCE page.
- Moving a bound of a linked pad unlinks it at once with a status line, not a refusal:
  the hand asked for "edit only this fragment".

## Overlaps

- 04e: its "unlinked pad" is the `pad` source here; its label `amen ·` gives way to
  `amen · pad`. Its "the selected slice says where it lies" is the "pads under a slice"
  here. Update 04e when either is implemented first.
- 21: the parameter pages grow a third pad page; 21 archives first.

## As built (task 3)

- The value keys move a bound by the fine step on a tap (1 ms) and the coarse one when
  held (10 ms), as every value since 21; the source steps one place per press, it does
  not wrap. The knobs: k1 the source over its list, k2 and k3 the bounds in the file.
- The pads screen has its own undo for the source: a change of the source and the first
  move that unlinks a pad are one step each; moves of a pad that is already `pad` are
  not undone one by one.
- The SOURCE page takes the rows of ENV in the panel, so the panel keeps its height.
- A pad that is `slice N` of a file that was never opened in this session finds its
  cuts in the session (the pending chops), so the source list is right before F3.

## Open questions

- The words: `all`, `slice 3`, `pad` follow the MPC (default); the owner may ask for
  others.
- The pads under a slice: the key letters with the bank (`2W`) by default; to look at
  live.
