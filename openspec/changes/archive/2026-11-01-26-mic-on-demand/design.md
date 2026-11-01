# Design 26

## Model

- The routing takes one more input: whether the microphone is wanted now. The app wants
  it while a microphone recording runs or the settings screen is open; otherwise the
  routing removes the microphone links and adds none. The chosen input device is still
  picked and named, so "No microphone is connected" keeps working without a link.
- A recording links first, then starts the recorder: the first few milliseconds may be
  silent while the link settles; the recording keeps its 5 ms edge fades.

## Decisions (with the owner, 2026-10-30)

- Linked only on demand, no "always linked" setting: the fewest driver handovers.
- F12 keeps the meter: the audio-settings scenario "Microphone" stays true.

## Known limits

- The driver handover still happens at the start and the end of a microphone recording
  and on F12. If the sound breaks there, the suspect is confirmed and the next step is
  the program's own clock choice, a later talk.
- The input meter outside F12 and the recording rests at zero.

## Open questions

- Whether this cures the breaks is known only after days of use; the owner reports a
  break with its moment, the agent captures the graph before any repair.
