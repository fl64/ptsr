# 61 — Ctrl+Z on the pads undoes the last change

## Why

The owner (2026-11-01): pressed `Ctrl+A` many times, saw something change, pressed
`Ctrl+Z` and nothing came back — "as I see it, Ctrl+Z undoes the previous action, doesn't
it?". On the pads screen `Ctrl+Z` undid only a change of the pad source (27); the volume,
the pitch, the envelope, the filter, the effects, the mode, a swap, a clear had no undo. A
change made with the scope on "all pads" touches 128 pads and could not be taken back.

## What changes

- `Ctrl+Z` on the pads and files screens undoes the last change of the pads, one step a
  press: a parameter (volume, pitch, envelope, filter, drive, sends), the mode, a swap, a
  clear, a file put on a pad, the source (27), a shared effect slot (56).
- A knob turn or a run of value keys on one parameter is one step, on every pad it touched.
- The status line says what came back: `undo: volume on 16 pads`; with nothing left,
  `nothing to undo`.
- What changes no pad is not a step: the selection, the bank, the scope (`Ctrl+A`), the
  pages, the marks.

## Not in this change

- Redo (`Ctrl+Shift+Z`): later if needed.
- The chop screen keeps its own undo of the cuts.
