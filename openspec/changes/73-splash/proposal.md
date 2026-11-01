# 73 — The splash screen

## Why

The owner wants the ptsr logo at every start (2026-11-01).

## What changes

- At start, ptsr shows a splash screen for 6.4 seconds: the logo, `ptsr - PT's SampleR`, the version, what loads now, and `any key: start`.
- Any key or controller event closes the splash and does its own action at once: a pad plays, F3 opens the chop screen.
- Loading goes on under the splash. After it, the screen of the last session opens.
