# 25 — The recording shows on the strip

## Why

The owner (2026-10-30): while recording, the strip above the pads should show the sound
being recorded. Now it shows only ptsr's own output: a microphone recording leaves it
flat, an output recording hides the other programs that go into the file.

## What changes

- **The strip shows the recording**: while a recording runs (microphone or output), the
  strip above the pad grid draws the level of what goes into the file, newest at the
  right, on the same decibel scale, in the colour of the `● rec` mark. (Owner: always
  while recording, in the colour of the recording message.)
- It wins over the pad edit envelope: the edit envelope comes back when the recording
  stops; outside the edit the live output comes back.

## Out of scope

The strip on other screens (there is none now); a view of the finished take.

## How to verify

By eye: hold `Ctrl+R` and speak — the strip runs with the voice in the `● rec` colour;
release — the output strip comes back. `Ctrl+T` with music in a browser — the strip shows
the music. Open the pad edit, record — the strip shows the recording, then the envelope
again. Tests: render tests with a fake recording history for each scenario.
