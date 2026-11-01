# Design 55

## Decisions (owner, 2026-11-01)

- Glyphs on the frame plus the panel (picked from four mockups: glyphs and panel, a text
  line in the cell, the panel only, a frame colour).
- The glyph set: shapes (picked from shapes, music signs, letters).

```
┌E─────────── ↕◣∿◻  ┐┌R────────────── ∿ ┐
│ kick               ││ snare              │
│ one-shot +3        ││ gate               │
└────────────────────┘└────────────────────┘

─ TONE ● ─────────
 volume  +3 dB        bright
 pitch   -5 st        bright
─ ENV ────────────    dim, nothing changed
─ FILTER ● ───────
 filter  low-pass     bright
```

- A default: the value a fresh pad gets (pitch 0, attack, decay and release 0 ms, the
  filter off, cutoff and reso as a fresh pad has them); the start and end at the bounds
  their source gives (the whole file, the slice). The ENV and FILTER glyphs light on any
  changed parameter of the block.
- Plain Unicode glyphs, one cell wide in any monospace font: no emoji (`✂` breaks the frame
  width in some terminals).
- The glyphs stand right on the top frame, in the order pitch, envelope, filter, bounds;
  in a cell too narrow for them the key letter and the glyphs win over the frame line.

## Implementation notes

- The glyphs stand on the pads and files screens. The chop screen draws its own marks in
  the pad cells (slice lengths, dim and dashed pads, 36, 46) and shows no glyphs.
- The volume off 0 dB counts as a change in the panel (bright, `●` on TONE) but gives no
  glyph: the mode line already shows it.

- The owner (2026-11-01), seeing `● FILTER` with the filter off and the cutoff at 1.1 kHz:
  "if the filter is off, take it away". The filter counts as set only when it is on.

## Open questions

- The owner said "glyphs": Nerd Font icons (real pictograms) need a patched font in the
  terminal. Default: plain Unicode, as picked from the mockups.

## Decisions (owner, 2026-11-01, later)

- `◻` in place of `[]`, and it marks a pad that plays a part of the file (a slice or an own
  piece), not a moved bound: after a fresh chop (42) every laid-out pad turns `pad`, a
  "moved bound" glyph would light on all of them. A hand trim and a slice look the same on
  the frame; the panel tells them apart (`slice 3` or `pad`).
- `◻` (U+25FB) and not `□`: `□` is ambiguous width, some terminals draw it two cells wide.
