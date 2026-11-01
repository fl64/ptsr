# Design 61

## Decisions (owner, 2026-11-01)

- "Ctrl+Z undoes the previous action": one undo for every change of the pads ("ok" to the
  proposal); no redo for now.

## Defaults taken by the agent

- A step keeps the pads it touched as they were before (their whole settings), so an undo
  puts back the pad, its sound and its engine state with one rule; the engine hears the
  restored settings again.
- One step for a run: changes of the same parameter on the same pads, less than 1 s
  apart, join the step before; a knob turn sends many small changes.
- The history holds the last 100 steps, in memory only: a restart starts it empty.
- The source undo of 27 becomes a step of this history.
- A file put on a pad comes back as the old file of that pad; the sample is loaded again
  if it was dropped.

## Open questions

None yet.
