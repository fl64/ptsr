# 77 — Pad glyphs that keep the frame

## Why

The owner (2026-11-02, a screenshot): the mute group `①` on the pad frame covers the
corner of the frame. `①` and `◣` have an ambiguous width in Unicode (East Asian Width
"A"): the program gives them one cell, the terminal font can draw them wider.

## What changes

- The envelope glyph `◣` becomes `⩘` (a slope up and down, East Asian Width "N").
- The pitch glyph `↕` becomes `⇕` (the same up-down arrow, width "N"): the owner's
  screenshot showed `↕` over its neighbour too.
- The mute group leaves the frame for the mode line under the name: `· mute 2`.
- The mode on that line is a glyph: `⇥` one-shot, `⇢` gate (BACKLOG row 17, the mode
  part): `⇥ +1 · mute 2`. The panel keeps the word.
- F1 names the new glyphs.

## Out of scope

- Fewer glyphs on a pad, or a space between them.

## How to verify

1. Put pad Q into mute group 1 (`Enter`, `K` to `mute`, `L`). Close the edit. Make sure
   that the line under its name says `⇥ · mute 1` and the frame has no digit. Press `H`:
   the line says `⇢ · mute 1`.
2. Set an attack on a pad. Make sure that its frame shows `⩘` and the corner stands in
   place.
3. Set the pitch to +1. Make sure that the frame shows `⇕` and it does not cover `⩘`.
4. Press F1. Make sure that it names `⇕` pitch, `⩘` envelope, `⇥` one-shot, `⇢` gate and
   `mute N`.
