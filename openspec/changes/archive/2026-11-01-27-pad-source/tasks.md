# Tasks — 27

- [x] 1. The pad source in the model: `All`, `Slice`, `Own`, derived from today's link and region on load, no session format change; unit tests of the derivation and of the renumbering after a delete and a new cut; commit.
- [x] 2. The pad panel names the source (`break.wav · slice 3`, `· all`, `· pad`); render tests for the three; commit.
- [x] 3. The SOURCE page in the pad edit: source, start, end on knobs 1–3 and the value keys; a bound move on a linked pad makes it `pad`, the status says so, undo links it back; recorded-event tests for every scenario; commit.
- [x] 4. The chop screen: the header "cuts of <file>", the pads under each slice, the slice of the pad F3 came from selected; render and recorded-event tests; commit.
- [x] 5. User stories (`docs/USER-STORIES.md`, the pad panel, the SOURCE page, "cuts of"); the help line names the SOURCE page and the picked bound (test, and seen in tmux 2026-10-31: `EDIT  I K Tab SOURCE  J L start …`); commit.
- [ ] 6. Live check with the owner; commit if anything changes.
