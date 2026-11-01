# 32 — Hide the pads, the strip takes their rows

## Why

The owner (2026-10-31), after the one layout (31): "when I need it I make the pads smaller
and the envelope taller"; then "maybe just hide the pads in that layout". It has to work
everywhere the pads are.

## What changes

- **`Ctrl+O` hides and shows the pads** on the pads (F2), files (F6) and chop (F3)
  screens. Hidden, the strip takes the rows of the grid.
- **One line stays instead of the grid**: the bank and the selected pad, as
  `bank 1 ⢀⠂ · pad W kick`; it flashes on a hit. F6 still says where `Enter` puts the file,
  a hit still shows it landed.
- **The assign mode of F3 shows the grid anyway**: the plan cannot be checked without
  it; when the assign mode closes, the grid hides again.
- **The choice survives a restart**: it is kept in the `[ui]` section of the config.
- **The help line names the state**: `Ctrl+O pads: shown` / `pads: hidden`.

## Out of scope

Compact pads (dropped for hiding); the preview playhead on F6 (33).

## How to verify

F2, `Ctrl+O`: the grid goes, the wave takes its rows, the line says the bank and the
pad; hit W — the line flashes and names W. F6: `Enter` on a file — the file lands on
the pad the line names. F3: the wave is tall; `Enter` (assign) — the grid comes back,
`Esc` — it hides again. Restart — the pads are still hidden; `Ctrl+O` — back.
