# 33 — The preview shows where it plays

## Why

The owner (2026-10-31): "while a sample is previewed it would be good to show a running
cursor where the track plays now".

## What changes

- **A running line on the file wave (F6)**: while the preview plays, a thin green line
  runs over the wave in the strip at the place that sounds, as the playhead on the chop
  screen; it is gone when the preview stops or ends.
- The engine already reports where the preview plays (the chop playhead uses it); F6
  reads the same position.

## Out of scope

Seeking in the preview by moving the line; the playhead on F2.

## How to verify

F6 on a long file: the green line runs from the left edge to the right one in the time
the file plays; move to the next file — the line starts again from the left; `Space` —
the line is gone. Tests: the drawn strip with the line at a given position, the app
gives the position only while the preview plays.
