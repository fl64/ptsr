# Design 66

## Decisions (owner, 2026-11-01)

- "Yes" to the findings 1, 3 and 4 of the UX check as one change; finding 5 (the config path
  in F12) waits, finding 6 (the kitty notice) is not reproduced.

## Defaults taken by the agent

- The folder is read again on every entry to F6, not on a timer: the owner looks when they
  comes to the screen, and a read of a folder is quick.
- `Esc` on the files screen with a filter on clears the filter first, as `Esc` drops the
  marks first on the pads.

## Open questions

None yet.
