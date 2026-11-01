# Design — 06c

## Bank fill glyphs

Two braille glyphs per bank, side by side: each glyph is 2 dot-columns × 4 dot-rows, so two glyphs give a 4×4 dot grid that mirrors the physical pad grid of that bank (rows 1234 / QWER / ASDF / ZXCV top to bottom, columns left to right). A lit dot = the pad holds a sound (a whole file or a slice). The current bank keeps its brackets; empty banks show blank glyphs, which keeps the row aligned.

- Drawn in the existing bank row component, so every screen that shows the row (pads, files, chop, settings tabs) gets it for free.
- Updates come from the same session/pad state the pad grid reads; no new state.
- A pad marked "missing file" still counts as filled: the assignment exists, the pads screen names the problem.

## Three-row envelope preview

The preview strip of the selected file on the files screen becomes three text rows. Each column keeps its braille column structure; amplitude is split across the three rows (each row covers a third of the vertical range, each braille glyph still holds 4 dot-rows, so the effective vertical resolution is 12 dot-rows). The file length label stays on the first row, left, where it is now.

## Messages and the hint

One status line, two kinds of content:

- **Action results** (placed, replaced, saved) — as now, they fade.
- **The first-steps hint** — shown while no pad holds a sample; a fading message may cover it for its short life, then the hint returns. Implementation: the hint is the fallback content of the status line when nothing newer is pending, not a one-shot message.

Audio-unavailable is not a status-line resident: the corner indicator carries the state, one plain message is emitted once, the raw error goes to the log. The F12 Audio row gets its own honest text when the backend is JACK and the server is down.

## Same-sound replace

"Same sound" = the pad plays the whole file and the same whole file is put again. A pad playing a slice of a file is not the same sound as the whole file: the question stays. Cheap check: same source file and same play range.

## Open questions

1. **Bank fill glyph shape.** Default taken: two braille glyphs mirroring the 4×4 pad grid. Alternatives: a filled-count "3/16" (precise, busier) or four dots in a row (loses the grid). Decided by walkthrough: braille grid shows *where* in the bank the sounds sit, which is what the owner asked for.
2. **Empty bank glyph.** Default: blank braille glyphs (keeps alignment). Alternative: a middle dot placeholder. Blank chosen — an empty bank needs no eye stop.
3. **Preview rows.** Default: exactly three. Alternatives: two (still cramped) or four (eats the list). Three chosen by the owner.
