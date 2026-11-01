# Design 33

## Decisions (with the owner, 2026-10-31)

- The line looks like the chop playhead: thin, green (`METER`), over every row of the
  strip.
- The position comes from the atomic the audio thread already writes for the preview
  (`preview_pos`); no new message between the threads.

## Open questions

None.
