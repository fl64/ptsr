# Design 36

## Model

One rule for pads and cuts: a pad follows the slice that starts at its cut. Moving a bound
or joining changes the slice and the pad hears it; a cut inside makes the slice shorter;
a cut that disappears (a region, a new chop) leaves the pad playing what it played.

## Decisions (owner, 2026-10-31, one question at a time)

- 1.1 real pads on the F3 grid; 2.1 dash; 3.1 bank row always; 4: the left part ("so it
  plays only the first part, from the first bound to the new cut"); 5: a dot in the bank
  color; 6.2 defaults on a new pad; 7 dropped (the pad stays linked).
- The listen loses the 16-key run through slices before a layout; the cursor plays the
  slice it enters (21) instead.

## After the first look (owner, 2026-10-31, a screenshot)

- "How do I tell where the slices are bound? it is not clear at all": the `●` marks were
  one character, half hidden by the cursor `!`. A slice on a pad now wears a plate in its
  bank color across its width with the pad name (1.1 of the owner's choice).

## The model stays free (owner, 2026-10-31)

- The owner gets lost "every time in this story" (a file with cuts, slice numbers, pads
  anywhere, the source pad elsewhere). A simpler model was offered — one break, one bank,
  slice N on pad N, as hardware samplers do — and declined: the free layout stays (any
  slice on any pad, in any bank, on two pads). The screen has to carry the clarity:
  the plates, the source mark, the real pads on the grid.

## Before the layout (owner, 2026-10-31: "let's try it like this for now")

- With the real pads on the grid the slices of a fresh chop could be heard only with the
  cursor. Until a slice of the file lies on a pad, the grid is the keyboard of slices of
  before (16 per bank page, the bank actions turn the pages); the first slice laid out
  turns it into the real pads.

## Open questions

None.
