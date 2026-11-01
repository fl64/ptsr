# Design 25

## Model

- The engine stats keep a second rolling history: the per-block peak of what the
  recorder writes (the microphone input or the output tap), filled in the audio callback
  next to the output history — preallocated, atomics only, nothing new in the callback.
- The pads view gets the recording history while a recording runs; the strip draws it
  with the colour of the `● rec` mark instead of the live output or the edit envelope.

## Decisions (with the owner, 2026-10-30)

- Always while recording, also over the pad edit envelope: "I am recording, so show it".
- The colour of the recording message (`● rec`, magenta): the strip says at a glance that
  it shows the recording.

## Open questions

- Other screens have no strip; "always" is taken as "on the pads screen, in every
  layer". The chop and files screens stay as they are unless the owner asks.
