## 1. Specification

- [x] Record the owner decisions, the audit causes, the route table and the corner cases.
- [ ] Get the owner review of this specification before code.

## 2. Control row and questions

- [ ] Make device pads 1 and 4 answer questions, pickers, Learn and Exit. Remove Panic from the panel.
- [ ] Show only `no` and `yes` on the panel behind a question.
- [ ] Move Help tabs to LEFT and RIGHT.
- [ ] Test each scenario with keyboard events and raw MIDI through the real profile. Include corner cases E1–E3 and E8.

## 3. Targets on the playing layer

- [ ] Place items and Assign slices with first-layer hits on all 16 pads. Show mode commands during a hold.
- [ ] Mark the source, the occupied pads and the planned slices on the grid.
- [ ] Name the pads in placement results.
- [ ] Test with raw MIDI, both banks, `Hold panel` and `Off`. Include corner cases E5–E7.

## 4. Screen text

- [ ] Show device pad numbers with an SMC-PAD connection, with keys in the corners.
- [ ] Show knob numbers on rows and use the same names in rows and the knob strip.
- [ ] Keep the screen name in the panel title and stop badge overlap.
- [ ] Add the mode line. Make hints, the mode line and answer rows follow the last input device. Include corner case E4.
- [ ] Add the refusal notice for panel actions that cannot act.
- [ ] Compare the screen catalogue at 80×26 and 120×40.

## 5. Documents and checks

- [ ] Update `docs/CONTROLLER-PATHS.md` with all 40 stories, `docs/SMC-PAD.md`, `docs/VISION.md` and `docs/plans/SMC-PAD-INTEGRATION.md`.
- [ ] Walk the "What Changes" list on the screen and report each item that is not done.
- [ ] Run format, Clippy, changed tests and the fast checks. Give the owner the build and the checks.
