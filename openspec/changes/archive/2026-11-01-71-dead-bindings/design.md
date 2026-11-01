# Design 71

## Decisions (owner, 2026-11-01)

- "Remove the dead ones if they are not planned." Nothing of the six is planned.

## Defaults taken by the agent

- The command palette stays out until a change makes it; `Ctrl+K` is free meanwhile.
- A name counts as an action when the program knows it as a command or as a knob
  parameter.
- The owner's config is cleaned once: the dead MIDI bindings are dropped and the `[midi]`
  section is saved; the log lists each dropped binding, the status gives the count. The
  next start finds nothing to drop and says nothing.
- An unknown action in the owner's keymap (64) is now named on the status line, not
  skipped quietly as the design of 64 said: a typo should be seen.

## Open questions

None.
