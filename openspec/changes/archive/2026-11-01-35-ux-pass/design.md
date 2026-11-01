# Design 35

## Decisions (owner: "ok" to the recommended options, 2026-10-31)

- `J` `L` in the assign mode step the slice (they are the cursor keys outside it: the
  same hand, the same direction). The cursor goes to the slice start, as the slice pick
  does, so the selected slice and the cursor never part; the cuts stay.
- The region: pads of slices outside become `Own` with the bounds they played, as a cut
  inside a slice on a pad already does (04e).
- The session gets an optional `bank` (serde default 0): an old session opens on bank 1,
  no migration.
- "Empty" project samples: no audio files and no folders in it.

## Open questions

None.
