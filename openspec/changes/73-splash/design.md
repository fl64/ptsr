# Design

## Decisions

- The logo is the owner's text, byte-exact (4 lines of 10 characters, trailing spaces kept).
- The splash lives in the wave strip above the pad grid and stays until the first sound
  (owner, 2026-11-01): a note that plays or a recording that starts, whichever comes first.
  The 6.4 s timer and the `any key: start` line are gone: nothing blocks the keys.
- The splash shows only on the pads screen. Other screens show their own content. A return
  to the pads screen before the first sound shows the splash again (owner, 2026-11-02).
- The version comes from the build (`CARGO_PKG_VERSION`).
- The first big box and the full-width box came before this decision; the strip replaced
  them (owner: "draw it where the wave shows, until the wave actually goes").

## Defaults taken by the agent

- A hit on a pad with no sound (an empty pad, a sample still loading, a file that did not
  load) does not end the splash: no sound went.
- The pad edit views still take the strip while the splash shows (they are editing aids,
  not the output wave); the splash replaces only the plain output wave.
- The splash ends on the app side when the note goes to the engine, not by watching the
  output history: the event is exact and testable.
- The program's own MIDI port notes (`Connected`, `Disconnected`) are bookkeeping, not
  owner input; they never end anything.

## Open questions

- The logo in the full help (F1) too? Default: no, only on the splash.
