## Decisions

Use the same tab transition for Tab and SMC-PAD LEFT and RIGHT.
On entry to Paths, keep a valid root row. If the previous row is too high, select
the last root. On Input, row controls leave the stored row in place.

Process folder picker input before Settings input. This keeps the distinct
navigation roles of the arrow keys, E1, LEFT and RIGHT inside the picker.

## Edge cases

- Audio to Input with Click level selected. Cover both input devices. Preserve
  the audio config and the stored row.
- Audio or MIDI to Paths with an invalid row. Cover both entry directions.
  The visible root and the root that the picker opens must match.
- Paths with either root selected. E2 and the value keys must preserve the
  audio config and both roots.
- Open folder picker. Use the existing tests for navigation, select and cancel.

## Open questions

None.
