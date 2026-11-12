# Design

## Decisions

- **Parts, not words.** The App already joins the slice line with ` · `. The drawing splits it at ` · ` and packs whole parts into rows of the column width. The first row keeps the `EDIT · BOUNDS` badge when the edit is open.
- **Long parts.** A part wider than the column wraps by words, as before. So a narrow terminal loses no text.

## Open questions

None.
