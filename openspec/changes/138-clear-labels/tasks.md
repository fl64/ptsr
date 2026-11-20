## 1. Specification

- [x] Record the owner decisions and the label table.

## 2. Labels

- [x] Rename the shipped panel labels. Update the label glossary and the shipped table test.
- [x] Show `show pads` while the pads are hidden. Name the hidden grid in F1 Help.
- [ ] Dim `reset trim` on a whole region. Show a notice for each press. Test both scenarios with keys and raw MIDI.
- [x] Rename the Chop knob labels and their full names on the line of the last knob.

## 3. Dim and destructive cells

- [ ] Dim each cell of the design table while it cannot act. Explain a press on a dim cell.
- [ ] Draw destructive cells in the warning colour.
- [ ] Ask before hand cuts, a pad sound or a learned control goes. Track hand cuts in the chop.
- [ ] Name the change and the undo after each destructive action. Add an undo step to `level to peak`.
- [ ] Test every new scenario with keys and raw MIDI.

## 4. Remove all cuts

- [ ] Add `chop.cuts_clear` with the P6 cell and the Delete key. Dim it without cuts. Ask, then name the undo.
- [ ] Allow eight cells a label line in the loader, the glossary and the drawing.
- [ ] Add the route to the route table. Test both scenarios with keys and raw MIDI.

## 5. Checks

- [ ] Check every panel view at 80×26 and 120×40 in the screen catalogue.
- [ ] Run format, Clippy, changed tests and the fast checks.
- [ ] Recommend the walk level to the owner.
