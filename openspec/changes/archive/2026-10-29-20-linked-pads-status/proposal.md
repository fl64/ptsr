# 20 — The status says which edits change the pads

## Why

Pads with slices follow the cuts: a moved or deleted cut, a new cut inside a slice, the region edges — all change what laid-out pads play. The owner asked when the cuts change the pads; today only a new equal-parts chop says so in the status, the other edits change the pads silently.

## What changes

- After an edit on the chop screen that changes the sound of laid-out pads, the status says how many: "2 pads changed; Ctrl+Z returns". After undo: "2 pads changed back".
- Equal parts keep their own message (the pads are unlinked, not changed).

## How to verify

1. Lay 8 slices out on pads, F3 on one of them, move a cut with `.`: the status says "2 pads changed; Ctrl+Z returns".
2. `Ctrl+Z`: "2 pads changed back".
3. A sample with no slices on pads: the same edits say nothing.
