# 76 — Marks are the scope, and a pad can be reset

## Why

The owner (2026-11-02): "I can mark several pads, but what to do with them next is
unclear" (BACKLOG row 20), and "I did not understand how `Ctrl+A` works". Today two ways
pick the pads that a change hits: the marks, and `Ctrl+A` that steps an invisible scope
(pad, bank, all pads). A forgotten `bank 2` silently sends the next change to 16 pads.
Also, a pad edit cannot be put back to clean: every parameter must be turned back by hand.

## What changes

- One way to pick pads: the marks. The scope is the marked pads, or the selected pad when
  no pad is marked. The scope steps "bank" and "all pads" are gone.
- `Ctrl+A` marks the 16 pads of the open bank and turns marking on. In a bank where all 16
  pads are marked, it unmarks them.
- `Ctrl+Shift+A` marks all 128 pads. Ghostty takes this combo for "select all" by default;
  the README tells how to free it.
- A marked pad is easy to see: a dark yellow fill and a bright yellow frame.
- The bank row shows the marks: a bank with marks shows its marked pads as yellow dots, in
  every bank, open or not.
- `Backspace` in the pad edit resets the pad to clean: all its settings come back to a
  freshly loaded pad (volume 0 dB, one-shot, pitch 0, envelope, filter, drive, sends, mute
  group off). The sound and its bounds (whole file, slice or own piece) stay. It acts on
  the scope and asks first: `Enter` resets, `Esc` keeps. `Ctrl+Z` brings all back.
- `Esc` in the pad edit closes the edit and keeps the marks; a second `Esc` drops them.

## Out of scope

- Copying the settings of one pad to other pads, moving or copying a group of pads with
  their sounds (BACKLOG row 20 keeps them).
- A reset of one block only (only the filter, only the envelope).
- `Backspace` outside the edit stays as it is: it clears the selected pad.

## How to verify

1. On F2, press `Ctrl+A`. All 16 pads of the bank turn dark yellow with a yellow frame.
2. Hit two pads: they lose the yellow. Go to bank 5 (`Ctrl+5`) and hit three pads: they
   turn yellow. In the bank row, bank 2 and bank 5 show yellow dots where the marks are.
3. Press `Enter`, turn the volume down. Every marked pad in both banks is quieter.
4. Still in the edit, press `Backspace`. A question asks `Reset 17 pads to clean?`. Press
   `Enter`. Listen: the pads sound as freshly loaded, with the same cuts.
5. Press `Ctrl+Z`. Listen: the pads sound as before the reset.
6. Press `Esc`: the edit closes, the marks stay. `Esc` again: the marks are gone.
7. With `keybind = ctrl+shift+a=unbind` in the Ghostty config, press `Ctrl+Shift+A`: every
   bank in the bank row shows all dots yellow.
