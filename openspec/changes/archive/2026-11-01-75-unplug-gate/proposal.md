# 75 — A pulled-out controller releases its gate pads

## Why

The scenario "Controller pulled out during play" said "the sound does not stop". The program releases the gate pads held by the lost device: they fade over their release. Else a gate pad sounds forever, because nothing can release it. The owner (2026-11-01) chose the program behavior.

## What changes

Only the spec text. The code already does this (`dispatch` in `crates/app/src/midi.rs`, test `controller_pulled_out_during_play`).
