# Design 51

## Decisions (owner, 2026-10-31)

- 1 of 3 ("Q1.1"): strong hits only, the start of the attack, marks on the wave. Weighed:
  the top of the hit (Q1.2), cuts the attack; marks only (Q1.3), as many stops as now.
- "The auto-chop too": `Ctrl+Y` and the cursor share one threshold, so the marks are the
  cuts `Ctrl+Y` makes; `Ctrl+Y` keeps its steps (−6, −12 … −30 dB).
- −12 dB as the default threshold: on the owner's recordings it passes the swells inside
  one sound (up to 11 of 37 hits fell below it). assumption: right for breaks too; the
  owner judges by eye and ear.

- "Set" is its own flag, not a threshold of 0 dB (found while coding): the knob at 0 dB
  keeps only the loudest hit, as 04d "The threshold on a knob" says.

## Archive order

"Cursor modes" stands on 43: 51 is archived after 43.

## Open questions

None yet.
