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
- D7. `Ctrl+Shift+A` marks all 128 pads. Ghostty binds `ctrl+shift+a=select_all` by
  default (checked with `ghostty +list-keybinds`), so the README tells the owner to add
  `keybind = ctrl+shift+a=unbind`. A terminal without the kitty keyboard protocol sends
  `Ctrl+Shift+A` as `Ctrl+A`: there it marks the bank.

## Defaults taken by the agent (the owner did not object, 2026-11-02)

- M1. `Ctrl+A` in a bank whose 16 pads are all marked unmarks them; marking stays on.
- M2. `Esc` in the pad edit closes the edit and keeps the marks; `Esc` outside the edit
  drops them, as today.
- M3. In the edit, the panel shows the values of the selected pad (the last hit). A change
  moves every pad of the scope by the same step, as today.
- M4. Copying settings and moving a group stay in BACKLOG row 20.

## Defaults taken by the agent (new)

- The actions get names that say what they do: `pad.mark_bank` (`Ctrl+A`) and
  `pad.mark_all` (`Ctrl+Shift+A`). The old name `pad.scope_next` in a user keymap or a
  learned MIDI binding is read as `pad.mark_bank`, so no file of the owner breaks.
- `Ctrl+Shift+A` when all 128 pads are marked unmarks them all, the same way as M1.
- The reset question names the count: `reset pad Q to clean? Enter reset · Esc keep` or
  `reset 17 pads to clean? ...`. While it asks, pad hits still sound and do not
  answer it; any other command key keeps the pads as they are, as `Esc` does.
- The reset of a pad already clean is still a step; the status says `reset: 1 pad`.
- An empty pad in the scope is skipped: it has nothing to reset.

## Open questions

- None yet.
