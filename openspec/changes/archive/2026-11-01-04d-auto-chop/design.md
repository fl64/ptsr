# Design 04d

Done after 04c (owner decision, 2026-10-28): cuts belong to a file and the pads are
linked to them; auto-chop replaces the cuts the same way `Y` does (linked pads unlink,
undo relinks).

## Detector

Runs off the audio thread on the loaded sample once per file; the onsets (with their
strength) are kept with the chop. Changing the threshold only filters the kept onsets,
so cuts follow the knob and the key at once. The level-rise detector in `chop.rs`
(`find_hits`, used by the hit-to-hit cursor) is the baseline the comparison starts from.

## The threshold

- A hit stays when its strength is within the threshold below the loudest hit of the
  region. `Ctrl+Y` steps the threshold: −6, −12, −18, −24, −30 dB, then −6 again.
- The knob k11 sets the same threshold continuously over 0…−40 dB; the status names the
  threshold and the number of cuts.
- Hits closer than 50 ms merge into one, as in `find_hits`.

## Snapping

When cutting in time, the cut snaps to the nearest onset within ±90 ms. `N` is the one
switch: since 21 it steps the cursor hit to hit; on, it also snaps the cuts; off, the
cuts land where they are pressed.

## Hit strength

`find_hits` returns each hit with its strength (the peak of its block); the threshold of
`Ctrl+Y` and of the knob keeps the hits within the threshold below the loudest hit of the
region. The hits are found once per file, off the audio thread, and kept with the chop.

## Decisions (with the owner, 2026-10-30)

- `Ctrl+Y` next to `Y`: "the more you press, the finer it chops", as with equal parts.
  `Ctrl+Y` was bound to `edit.redo`, which the program never had; a redo, if ever
  needed, takes `Ctrl+Shift+Z`.
- The tempo grid leaves 04d: a change of its own later.
- The detector is picked by ear on the owner's breaks before the code.

## Detector comparison (task 0.2, prepared 2026-10-30)

Renders in `renders/` (not in git), the break at 60 % with a 2 ms click on every hit:
`04d-compare-amen-own.wav`, `04d-compare-amen-aubio.wav` (the looperman "amen to that"
loop, 5.65 s), `04d-compare-pressure-own.wav`, `04d-compare-pressure-aubio.wav`
(Gabriel Saban, "Growing Pressure", cut 1, 93.7 s). "own" is `find_hits` of `chop.rs`
ported line by line; aubio 0.4.9 runs `default` (HFC), window 512, hop 256, minimum gap
50 ms, silence −40 dB. The tool is `target/04d-compare/compare.c` (scratch, not in git).

- Amen: own 43 hits, aubio 37; most hits agree within a few ms; they part on the quiet
  notes (own 0.617 and 0.685 s where aubio has one at 0.699 s).
- Growing Pressure: own 447, aubio 551; aubio keeps a steady ~0.13 s grid in dense parts
  where own leaves gaps.
- Decided (owner, 2026-10-30: "pick the optimal and best one, we change it later if
  needed"): the own detector for now. On breaks it is close to aubio, it is already in
  the program, it needs no dependency; it stays one function, so aubio can replace it
  later without touching the keys. Dense music is where aubio is ahead. aubio is GPL-3.0-or-later, compatible with
  the project; using it from Rust needs bindings (a crate or a few lines of FFI) — to be
  named in the proposal if aubio wins.

## Open questions

- Which breaks to compare on: the owner did not name them; default — 2–3 breaks from
  the samples root, one with ghost notes; the owner may swap them.
- The threshold steps and the knob range are to poke live.
