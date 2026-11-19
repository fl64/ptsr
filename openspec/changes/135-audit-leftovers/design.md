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

## Open questions

1. Labels `rows` and `tap` in A1. Default: they stay. A rename needs the owner.
