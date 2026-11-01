# Design 23

## Model

- The files screen keeps its layout; only the meaning of `Enter` on a file changes, and
  the preview gains a default.
- "In hand" stays: it is the only way to put one file on several pads and to choose the
  pad after seeing the highlighted grid.

## Decisions (with the owner, 2026-10-30)

- `Enter` on a file = load to the selected pad and close the screen; `Ctrl+Enter` = take
  in hand; folders open as before.
- The preview starts off; `Space` toggles it, the help line names the state.

## Open questions

None.
