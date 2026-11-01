# Design 41

## Decisions (owner, 2026-10-31)

- Turned around from 23 (owner: "so, the other way round"): `Enter` = in hand, `Ctrl+Enter` =
  onto the selected pad.
- No new code path: the keymap swaps `browser.hold` and `browser.load`; the hold already
  opens a folder and, with a file in hand, puts it on the selected pad.

## Open questions

None.
