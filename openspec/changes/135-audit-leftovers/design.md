# Design

## Source

Codex reports of 2026-11-19: 129 actions, 73 ok, 25 unclear labels, 8 without a screen
answer, 22 screen errors, 1 dead end. Change 134 fixed the dead end, the labels, the dim
cells and the Assign bank. This change takes the rest that is not about Chop flow (132).

## Decisions

- D1. A Chop notice goes to the notice row, not over the wave labels.
- D2. At 80×26 the mode line has priority over the hint columns. Apply and cancel come first.
- D3. A pending Learn binding on an SMC-PAD panel cell names the cell by its printed number and its current label.
- D4. The marking hint uses the words of the last input device, as other hints do since 134.
- D5. In the slice edit, the panel cells `cut -` (P15) and `end -` (P16) become `join left` and
  `join right`. The labels name the result, not a cut. The owner chose this on 2026-11-20 after
  `del left` did not say that a bound goes away. Keyboard keys stay unchanged.
- D6. While the Chop edit actions show on the panel, the wave marks the left and the right bound
  of the selected slice. The owner sees which bound goes away before the press.
- D7. `join left` on slice 1 and `join right` on the last slice are dim. A press shows
  «slice 1 has no slice on the left» or «slice 8 has no slice on the right». With one slice, both
  are dim and a press shows «nothing to join: one slice». Before this change the press did nothing.
- D8. After a join, a notice names the joined slices, for example «slices 2 and 3 joined».

## Open questions

1. Labels `rows` and `tap` in A1. Default: they stay. A rename needs the owner.
