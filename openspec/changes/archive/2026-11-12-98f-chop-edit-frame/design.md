# Design

## Decisions

- **Frame.** The slice edit frame has one page, `BOUNDS`, lit in the title as the open page of the pad edit. A narrow column drops `EDIT ·` first.
- **Rows.** `start` and `end` are the two rows that I/K selects. `length` and `cursor` are information rows, dim and never selected.
- **Values.** The App gives the four values with their units, as the slice line does now. The bounds and the cursor are in seconds, the length in ms.
- **Closed edit.** The right column keeps the slice line and the side list of 98d.

## Open questions

None.
