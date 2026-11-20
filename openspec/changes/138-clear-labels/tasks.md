## 1. Specification

- [x] Record the owner decisions Q1–Q28 and the walk findings that they close.
- [ ] Write the spec deltas for each modified capability.

## 2. Labels and layouts

- [x] Rename the shipped panel labels. Update the label glossary and the shipped table test.
- [x] Show `show pads` while the pads are hidden. Name the hidden grid in F1 Help.
- [x] Rename the Chop knob labels and their full names on the line of the last knob.
- [ ] Apply the D3 layouts. Remove `whole`, `undo A1 pass`, `learn`, `learn control` and `forget control` from the panels.
- [x] Allow eight cells a label line in the loader, the glossary and the drawing.

## 3. Cell looks

- [x] Draw orange cells and grey cells, and the current pad edit page brighter.
- [ ] Grey each cell of the D5 table while it cannot act. The cells of steps 5, 6 and 8 follow with their actions.
- [x] Explain a press on a grey cell with a notice.
- [x] Show each two-word label on two lines. Show the current state in the state cells. Rename `level to peak` to `norm`.

## 4. Destructive actions and undo

- [x] Track hand cuts in the chop. Ask before hand cuts go: equal parts, cut by hits, fewer hits, remove all cuts, the threshold knob.
- [x] Ask before `clear pad` on a pad with a sound. Remove false «This cannot be undone.» texts.
- [x] Name the result and the undo after each destructive action.
- [x] Make undo and redo notices say what came back on Chop, Pads and A1.

## 5. Chop

- [x] Add `remove all cuts` with the P13 cell and the Delete key.
- [x] Add `trim silence`. Ask when the chop has cuts.
- [x] Move `fewer hits` to Chop and `hear as pad` to the slice edit.

## 6. Normalize

- [ ] Add a saved sound gain per pad, applied before the pad volume in the engine.
- [ ] Make `normalize` set the gain of the pads that play the selected slice, with undo.
- [ ] Render before and after to `renders/`. Check level, clipping and clicks. Run the callback benchmark.

## 7. Modes and recording

- [ ] Keep the exit part of the mode line at 80×26. Add the D10 mode lines.
- [ ] Make PAD BANK + P1 leave the active mode first.
- [ ] Make STOP end the output recording on all screens.
- [ ] Fix the device words of the notices that the walk found. Record the other key texts in `docs/BACKLOG.md`.

## 8. Other screens

- [ ] Add the pad edit page cells. Make LEFT and RIGHT go around the pages.
- [ ] Show the Assign method. Ask before a method change drops placed slices. Grey `apply` until the start pad.
- [ ] Make `new project` work with no projects and name the project by date.
- [x] Show «Bindings: keyboard only (F10, Enter)» on F12 MIDI. Make forget, reset and the layout update ask.

## 9. Documents and checks

- [ ] Update `docs/CONTROLLER-PATHS.md`, `docs/plans/` and `docs/VISION.md` for the layouts and the binding exception.
- [ ] Add every new mode to the mode walker test. Test hints after SMC-PAD and key input at 80×26 and 120×40.
- [ ] Check every panel view in the screen catalogue.
- [ ] Run format, Clippy, the slow tests and the fast checks.
- [ ] Recommend the full walk to the owner.
