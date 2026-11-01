## 0. Before
- [x] 0.1 04c done (archived 2026-10-28)
- [x] 0.2 Hit detector (own `find_hits`, owner 2026-10-30; aubio later if needed): aubio vs own Rust detector; renders of 2–3 real breaks with a click on every hit found, one per detector, in `renders/`; the owner decides by ear; the licence of a new dependency checked

## 1. Hits
- [x] 1.1 `Ctrl+Y` chops by hits, stepping the threshold −6…−30 dB and over; one undo step per press; `edit.redo` unbound; recorded-event tests and an offline render with the automated checks
- [x] 1.2 The threshold on k11 (`chop.threshold`): cuts follow the knob at once; recorded-event tests
- [x] 1.3 Snapping of cuts to hits within 90 ms, on with the `N` hits mode of 21; test: the marker lands exactly on the transient

## 2. Verification
- [x] 2.1 `renders/04d-chopped-by-hits.wav`
- [x] 2.2 User stories and the help line
