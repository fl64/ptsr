# 77 — Pad glyphs that keep the frame

## Why

The owner (2026-11-02, a screenshot): the mute group `①` on the pad frame covers the
corner of the frame. `①` and `◣` have an ambiguous width in Unicode (East Asian Width
"A"): the program gives them one cell, the terminal font can draw them wider.

## What changes

- The envelope glyph `◣` becomes `⩘` (a slope up and down, East Asian Width "N").
- The mute group is its digit, dark on a light plate: `1`…`8` instead of `①`…`⑧`.
- F1 names the new glyphs.

## Out of scope

- The pitch glyph `↕` (also width "A") stays until the owner sees it cover the frame
  (owner, 2026-11-02: check first, change only if it breaks).

## How to verify

1. Put pad Q into mute group 1 (`Enter`, `K` to `mute`, `L`). Close the edit. Make sure
   that the frame shows `1` dark on a light plate, and that the right corner of the frame
   stands in place.
2. Set an attack on a pad. Make sure that its frame shows `⩘` and the corner stands in
   place.
3. Press F1. Make sure that it names `⩘` envelope and `1…8` mute group.
