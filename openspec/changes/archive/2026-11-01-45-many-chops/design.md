# Design 45

## Decisions (owner, 2026-10-31)

- 1 of 3 ("Q1.1"): several chops per file, each with its pads. Weighed: as an MPC, F3
  shows only the pad's own piece (Q1.2); one chop per file with a warning (Q1.3).
- The newest chop of a file is the one the files screen opens and the whole-file pad
  replaces; older chops live while a pad links to them and are dropped with their last
  pad.
- An older chop is identified by its epoch: pads already link by (cut id, epoch), so a
  chop is found from a pad without a new link format.
- Rebuild of a lost chop (projects saved before 45): the pads that link to the epoch give
  its cuts, every start and end inside the file; a start keeps the pad's link id, the
  other cuts get new ids; the range is the whole file.

## Archive order

"The cuts belong to the file" stands on 42: 45 is archived after 42.

## Open questions

- Telling the chops of one file apart on the chop screen. Default: the header says
  `cuts of break.wav · chop 2 of 3`.
