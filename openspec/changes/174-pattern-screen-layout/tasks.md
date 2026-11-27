## 1. Pattern screen

- [x] 1.1 Draw the Pattern screen in place of the pad grid and inspector while the Pads overview mode is on. Keep the `[screens.pads]` keymap section and MIDI context. F5, the screens view cell and learned `screen.pattern` open it.
- [x] 1.2 Draw the header line, the rows without bank frames, the bank rule and the full-width cells.
- [x] 1.3 Draw the three velocity glyphs, the velocity lane, the detail line and the read-only setup line.
- [x] 1.4 Draw the absent and empty states and the scrolled row list.
- [x] 1.5 Keep the setup panel, Step Edit, record, count-in cancel, erase, tap tempo, view and all-rows keys on the new screen.
- [x] 1.6 Show `F5 pattern` in the Pads lane without a pattern. Rename `A1` to `pattern` on the screen, the lane and the screens cell.
- [x] 1.7 Replace the old overview drawing and its catalogue entries.

## 2. Tests

- [x] 2.1 Update the tests of the modified scenarios. The scenario names do not change. The groups are:
  - rows, hidden hits, bank and view changes, hidden empty banks, cells and velocity levels.
  - position, selection, count-in cancel, step correction entry and position.
  - layout, playhead, absent and empty states, catalogue, strip, compact Rhythm, glossary and correction states.
  - help, cues, tap tempo, transport state, compact help and Setup.
  - F5 key, pad ownership, empty-bank key, browsing, pad edits, learned buttons and the screens view.
- [x] 2.2 Add a test for each new scenario. Name it after the scenario.
  The new scenarios are about pages, the velocity lane, the detail line, the setup line and the pattern keys.
- [x] 2.3 Replace the bank-border test of the removed requirement with the bank rule test.
- [x] 2.4 Run keyboard and raw MIDI paths for F5, the screens view cell, setup, record, Step Edit, erase and tap tempo.
- [x] 2.5 The mode walker in `crates/app/src/input_tests/controller_reach.rs` already enters the overview mode. The Pattern screen adds no new mode.
  Check hints after an SMC-PAD input and after a key at 80x26 and 120x40. Add the `pattern` cell label to the glossary test.
- [x] 2.6 Compare the ANSI catalogue with the base and check the files in the proposal.

## 3. Documents

- [x] 3.1 Update `docs/CONTROLLER-PATHS.md` names. The press counts do not change.
- [x] 3.2 Update `docs/VISION.md` section 17 for the new screen.
