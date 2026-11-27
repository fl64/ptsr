## Decisions

Use the same tab transition for Tab and SMC-PAD LEFT and RIGHT.
On entry to Paths, keep a valid root row. If the previous row is too high, select
the last root. On Input, row controls leave the stored row in place.

Process folder picker input before Settings input. This keeps the distinct
navigation roles of the arrow keys, E1, LEFT and RIGHT inside the picker.

The folder picker takes the work area. It gives the list enough rows to show the selection and keeps its controls visible.

## Edge cases

- Audio to Input with Click level selected. Cover both input devices. Preserve
  the audio config and the stored row.
- Audio or MIDI to Paths with an invalid row. Cover both entry directions.
  The visible root and the root that the picker opens must match.
- Paths with either root selected. E2 and the value keys must preserve the
  audio config and both roots.
- Open the picker with and without the panel. Check the selected folder and controls at 80x26 and 120x40.
- Move to the end of a long list. Keep the selection and controls visible.
- Cancel or pick the current folder. Keep the prior panel state and the existing input routes.

## Open questions

None.
