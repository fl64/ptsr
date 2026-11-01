# Design 64

## Decisions (owner, 2026-11-01)

- "Ok, the main thing is that it is handy": the user file first (64), the F12 tab on it
  next (65).

## Defaults taken by the agent

- The user file has the same sections as the shipped one (`[global]`, `[edit]`,
  `[screens.pads]`...). A key in a section replaces only that key of that section.
- `"none"` as the action unbinds a key, so a shipped key can be taken away.
- The dev copy `config/keymap.toml` stays the source the program is built with; it is no
  longer read at run time.
- The notice of a broken file reads `<path> line N: <what>; using the shipped keys`; a
  refused key reads `<path>: keymap [section] Q: pad keys cannot be commands`. `Shift+Q`
  gets the same words as `Q`.
- An action name the program does not know is skipped without a word, as the shipped file
  skips actions of screens still to come.

## Open questions

None yet.
