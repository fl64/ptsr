# Tasks

## 1. Commands

- [x] 1.1 Remove `pattern.clear_pad`, `pattern.merge`, the Alt+Backspace binding and the Merge row. Keep `pattern.clear` as a learnable GLOBAL button.
- [x] 1.2 Extend `pattern.erase` to whole-pad erasure on Pads outside a pass. Refuse on empty A1 and outside Pads. Give one Undo step per pad, silent presses and no click during playback.
- [x] 1.3 End whole-pad erasure on a pass start, a screen change, a project change, Esc and the toggle. End held erasure with the pass. Esc leaves erasure first.
- [x] 1.4 Clear A1 row with its hit count. Enter runs it and asks first. The change keys do nothing on it.
- [x] 1.5 Post result notices with counts. Write the erasure help line and F1 text.

## 2. Screen

- [x] 2.1 End the strip header with `2-9 = hits in a cell, one plays` while such a cell is visible.
- [x] 2.2 Show no mode glyph for one-shot pads.
- [x] 2.3 Show `ERASE` and `N hits` on the pads in whole-pad erasure. Add catalogue states at both sizes.
- [x] 2.4 Draw pattern controls in the right column in the pad panel style, without the pad panel under them.
- [x] 2.6 Frame pattern controls as the pad edit: edit color, `EDIT · PATTERN CLEAR` title, the selected block lit.
- [x] 2.5 While pattern controls are open, refuse pad, slice and effect commands. Edit knobs 1–3 turn Bars, Quantize and Strength.
- [x] 2.7 Strip header: `view`, `quantize` with strength, the open bank, no file name. Label the pad row `pad Q`.
- [x] 2.8 Show the event rows under the splash from the start.

## 3. Verification

- [ ] 3.1 Cover every scenario with keyboard and raw MIDI tests. Render whole-pad erasure during playback and check for clicks. Run `task check` and `task build`.
- [ ] 3.2 Owner check of the steps in the proposal.
