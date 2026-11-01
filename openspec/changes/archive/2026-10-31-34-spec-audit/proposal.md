# 34 — Spec audit: the texts as the program says them, a test for every scenario

## Why

The owner (2026-10-31): "are all the specs up to date? check, put in order, do what is
needed". An audit walked every scenario of the main specs and the active changes against
the code and the tests. Most are covered; some quote texts the program does not say, and
some are done in the code without a test (the project rule: every scenario has a test).

## What changes

- **Texts as the program says them** (no behavior change): the overwrite question
  `bank 2, pad Q: kick → slice 1` (as "Chop screen readability" already says), the file
  in hand with `Ctrl+Enter` (23 moved it there), the status `master -3 dB`.
- **A test for each scenario that had only code**: chopping (empty pad, hear the plan,
  all placed, undo a tap, steps that can be undone, the selected slice on the grid, the
  plain overwrite question), pad-assignment (change of mind, file is gone, swap while
  loading), pad-playback (new pad), screen-help (the last group), audio-engine (the
  status format, a buffer underrun), recording-side 26 (the input meter on F12).
- Fixed on the way, each in its own change's commit: the listen side of F3 named
  `PgUp/PgDn` and `Enter` instead of `Ctrl+1…8` and `P` (21); the Paths tab help showed
  `learn` and `clear` of the MIDI tab (11).

## Out of scope

Behavior the specs promise and the code does not do — those wait for the owner (see
design, Open questions). Scenarios that need the sound server or a controller in hand.

## How to verify

`task check` is green; every scenario named in the tasks has a test with its name.
