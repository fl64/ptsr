# Design 68

## Decisions (owner, 2026-11-01)

- "ok": all five items as proposed by the agent.
- The words "by hits" stay (the UX report found them unclear; the owner keeps them, not
  `auto: by hits (drum hits)`).

## Defaults taken by the agent

- The scope in the help line is text, as `mark: on` is, and rides on the mark item:
  `mark: off · scope: bank 2`. The `Ctrl+A` item is hidden in a kitty terminal until
  `Ctrl` is held (the rule of the held modifier), so the scope cannot stand there. The help line has no highlight for a single item,
  so a wide scope is not lit; the words carry it.
- `Ctrl+Y` keeps the threshold in its line, it is what the next `Ctrl+Y` steps:
  `by hits at -12 dB: 7 slices (were 16) · Ctrl+Z brings them back`. With one slice
  before (no cuts) the line ends after the number of slices: nothing was replaced.
- `▶` stands in the header whenever the chop screen sounds: `Space`, a slice played by a
  cursor jump or a pad. The assign mode keeps its own header.
- F1 explains the key releases only in a terminal without them: there it is the answer to
  the marker; elsewhere it is noise.
- The marker words refine the "no releases" marker of 22; the marker stays for the
  session as before.

## Open questions

None.
