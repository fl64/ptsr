# 73 — The splash screen

## Why

The owner wants the ptsr logo at every start (2026-11-01).

## What changes

- At start, the wave strip above the pad grid (the strip that shows the master output as a
  wave) shows a splash instead of the silent wave: the logo, `ptsr - PT's SampleR`, the
  version of the build and a loading line.
- The loading line names what loads now (the session, the number of samples), says `ready`
  when all is loaded, or names a file that did not load.
- The splash stays until the first sound: a note that plays (from the keys or the
  controller) or a recording that starts. Then the strip shows the wave and the splash
  never returns. Nothing blocks the keys meanwhile: a pad hit plays, F3 opens the chop
  screen.
