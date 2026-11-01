# Design 49

## Decisions (owner, 2026-10-31)

- A separate switch, `O` keeps picking what plays; `Space` starts and stops.
- The loop plays until `Space`, and goes on while the bounds move: the next turn plays the
  new bounds.

- The key: `Alt+O` (owner: "use Alt-O"), beside `O` that picks what plays. Weighed: `U`.
- A turn of the loop fades out and in over the short cut fade (5 ms) at its seam, as a
  slice end does, so the jump back does not click.

## Open questions

- The loop switch is kept while the chop screen is open and starts off on every open.
  Default: as said, not saved.
