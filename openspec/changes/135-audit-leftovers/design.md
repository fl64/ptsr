# Design

## Source

Codex reports of 2026-11-19: 129 actions, 73 ok, 25 unclear labels, 8 without a screen
answer, 22 screen errors, 1 dead end. Change 134 fixed the dead end, the labels, the dim
cells and the Assign bank. This change takes the rest that is not about Chop flow (132).

## Decisions

- D1. A Chop notice goes to the notice row, not over the wave labels. On 2026-11-20 the notice
  already used the notice row at both sizes, with and without the panel. The audit case did not
  reproduce, so this change adds a test that keeps the current layout.
- D2. At 80×26 the mode line has priority over the hint columns. Apply and cancel come first.
  The SMC-PAD mode line has about 40 cells there. When the line does not fit, the middle steps go
  first, then `PAD BANK + 4 apply · PAD BANK + 1 cancel` becomes `PAD BANK: 4 apply · 1 cancel`:
  «ASSIGN · PAD BANK: 4 apply · 1 cancel». Without an SMC-PAD the hint columns show Enter and Esc.
- D3. A pending Learn binding on an SMC-PAD panel cell names the cell by its printed number and its current label.
  This applies when the PAD BANK hold is off: the PAD BANK layer then sends its own notes. With the
  hold on, a cell is a hold of the playing note, and Learn already says that the panel layer keeps it.
- D4. The marking hint uses the words of the last input device, as other hints do since 134.
- D5. In the slice edit, the panel cells `cut -` (P15) and `end -` (P16) become `join left` and
  `join right`. The labels name the result, not a cut. The owner chose this on 2026-11-20 after
  `del left` did not say that a bound goes away. Keyboard keys stay unchanged.
- D6. While the Chop edit actions show on the panel, the wave marks the left and the right bound
  of the selected slice. The wave already draws these two bounds in magenta and the other cuts in
  cyan. This change keeps that and adds a test for it.
- D7. `join left` on slice 1 and `join right` on the last slice are dim. A press shows
  «slice 1 has no slice on the left» or «slice 8 has no slice on the right». With one slice, both
  are dim and a press shows «nothing to join: one slice». Before this change the press did nothing.
- D8. After a join, a notice names the joined slices, for example «slices 2 and 3 joined».
- D9. A panel cell has two text lines of 11 cells at 80×26. The label rule of seven cells stays
  for each line. A label longer than seven cells breaks at its first space, so `join left` shows
  «join» over «left». A short label such as `rec out` stays on one line. Change 138 uses this rule for the other labels.

## Open questions

1. Labels `rows` and `tap` in A1. Default: they stay. A rename needs the owner.
