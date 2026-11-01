# Tasks — 19

- [x] 1. Pitch in the engine: rate per voice with interpolation, glide on change; offline render `renders/19-pad-pitch.wav` (0, +12, −12) with the automatic checks and a length check; benchmark at 32 voices with pitch; commit.
- [x] 2. Pitch on the pad: state, session (old sessions open at 0), `Tab` picks the parameter, `-` `=` change it, panel shows both; tests; commit.
- [x] 3. Scope: `Ctrl+A` steps pad → bank → all, marking with `M`, framed marked pads, `Esc` drops marks, relative change with limits and the pop-up, mode for the scope; tests on recorded events; commit.
- [x] 4. Knobs: `pad.pitch` with soft takeover, both knobs move the scope by the delta; recorded-event tests; commit.
