# Design

## Decisions (owner, 2026-11-02)

- D1. `Ctrl+A` marks the whole open bank and turns marking on. The scope steps "bank" and
  "all pads" are gone: the marks are the only way to pick several pads.
- D2, D3. A marked pad has a dark yellow fill and a bright yellow frame; the name stays
  readable. A thin frame in one more color is not enough: the bank frames are coral,
  amber, olive and so on, and a yellow frame on bank 3 (olive) looks the same as no mark.
- D4. The bank row reuses the way a hit shows there (62): a bank with marks shows the
  marked pads as yellow dots in its glyphs, the open bank and the closed banks alike. A hit
  of the open bank still shows its white dots for the flash, then the yellow comes back.
  One glyph cell has one color, so while a bank has marks its glyphs show the marks, not
  the fill.
- D5, D6. `Backspace` in the pad edit resets every pad of the scope to a freshly loaded
  pad: volume 0 dB, one-shot, pitch 0, the envelope, the filter, the drive, the sends and
  the mute group at their defaults. The file and the source (whole file, slice, own piece
  with its bounds) stay. It asks first: `Enter` resets, `Esc` keeps. The reset is one undo
  step: one `Ctrl+Z` brings back to every pad its own settings.
- D7. No key marks all 128 pads (owner, 2026-11-02, after the live try: "Ctrl+Shift+A
  does not work, it selects the text on the screen, let us remove it"). `Ctrl+A` on each
  bank does it.
- D8. A plate at the top says that marking is on (owner, 2026-11-02: "a plate on top is
  missing that says mark mode").

## Defaults taken by the agent (the owner did not object, 2026-11-02)

- M1. `Ctrl+A` in a bank whose 16 pads are all marked unmarks them; marking stays on.
- M2. `Esc` in the pad edit closes the edit and keeps the marks; `Esc` outside the edit
  drops them, as today.
- M3. In the edit, the panel shows the values of the selected pad (the last hit). A change
  moves every pad of the scope by the same step, as today.
- M4. Copying settings and moving a group stay in BACKLOG row 20.

## Defaults taken by the agent (new)

- The action gets a name that says what it does: `pad.mark_bank` (`Ctrl+A`). The old name `pad.scope_next` in a user keymap or a
  learned MIDI binding is read as `pad.mark_bank`, so no file of the owner breaks.
- The reset question uses the question box of the other questions (the overwrite of a
  pad, the reset of a device): `Reset 17 pads to clean?`, what stays, and
  `Enter: reset · Esc: keep`. As in those questions, other keys (pad hits too) do nothing
  while it asks. While it asks, `Enter` and `Esc` are the keys of the pads screen, not of
  the edit layer, where `Enter` closes the edit.
- `Backspace` keeps one action, `pad.clear`: outside the edit it clears the selected pad,
  in the pad edit it asks for the reset. A separate binding in `[edit]` would take
  `Backspace` from the slice edit of the chop screen too.
- A reset that changes nothing (the pads are clean already) leaves no undo step.
- An empty pad in the scope is skipped: it has nothing to reset.

## Open questions

- The plate (D8): where and what it says. Default: on the title line of the pads screen,
  next to `F2 PADS`, black on yellow: `MARK · hit pads · 17 marked` while marking is on,
  `17 MARKED · Esc drops` while marking is off and pads stay marked; no plate when
  marking is off and nothing is marked.
