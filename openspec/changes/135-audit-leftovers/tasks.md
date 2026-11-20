## 1. Specification

- [x] Record the remaining audit findings.
- [x] Get the owner review of this specification before code.

## 2. Screen

- [ ] Move Chop notices to the notice row. Test both sizes.
- [ ] Give the mode line priority at 80×26. Test Assign.

## 3. Words

- [ ] Name the replaced panel cell in the Learn prompt. Test with raw MIDI.
- [ ] Use SMC-PAD words in the marking hint. Test after SMC-PAD input and after a key.

## 4. Checks

- [ ] Add each case to the mode walker test where it applies.
- [ ] Run format, Clippy, changed tests, the screen catalogue and the fast checks.

## 5. Join slices

- [x] Draw a panel label longer than seven cells in two lines at its first space. Check each line against the glossary and the seven-cell limit.
- [x] Rename the Chop edit cells to `join left` and `join right`. Add both to the label glossary.
- [x] Mark both bounds of the selected slice while the panel shows the Chop edit actions.
- [x] Dim a cell without a neighbour and explain a press. Name the joined slices after a join.
- [x] Test the four scenarios with keys and raw MIDI, at 80×26 and 120×40.
