# Design

## Decisions

- The logo is the owner's text, byte-exact (4 lines of 10 characters, trailing spaces kept).
- The time is 6.4 s (owner). A key closes it earlier and acts: pads always play (AGENTS.md).
- The version comes from the build (`CARGO_PKG_VERSION`).
- The splash is a box over the interface (owner, 2026-11-01), not a full-screen replacement: the screen of the session stays behind it and shows around it.

## Defaults taken by the agent

- The splash closes on owner input from a controller (a message, a pad hit or release). The program's own port notes (`Connected`, `Disconnected`) are bookkeeping and do not close it: two ports greet at start and killed the splash before the owner saw it.

## Open questions

- The logo in the full help (F1) too? Default: no, only on the splash.
