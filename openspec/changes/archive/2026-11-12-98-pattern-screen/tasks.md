# Tasks

## Working boundary

The owner approved 98 alongside unfinished 97 in the current working branch.
After the owner check on 2026-11-11, 98 covers only the rhythm view.
The step editor tasks moved to the deferred draft [98a step edit](../../../docs/plans/98a-step-edit.md).
The approval does not include publication, archive, merge, live devices or owner projects.

## 1. Event view model

- [x] 1.1 Add a checked App-side view model using existing playback compilation and raw-row identities.
  Cover all view divisions at every playback grid and strengths 0, 50 and 100.
  Test absent and empty A1 without mutation, hidden rows, loop seams and same-tick collisions.
- [x] 1.2 Add width-based page calculation: the most whole bars, then half a bar, then one beat.
  Cover every page for 1-bar, 2-bar and 64-bar loops at 80 and 120 columns. Keep triplet pages aligned to complete beats.
- [x] 1.3 Add runtime-only view selection with Alt+Up/Alt+Down and learned buttons on Pads.
  Order the views by cell size. Preserve owner overrides, both layouts and protocols, and existing input priority.

## 2. Visible rhythm on Pads

- [x] 2.1 Add the three-row event strip (header, all-pads row, selected pad's row) under the live output wave.
  Replace the wave only in a three-row strip. Preserve absent-A1 instructions, empty patterns, pad visibility and safety status at 80x26 and 120x40.
- [x] 2.2 Add event density, off-grid marks, dim empty cells, alternate beat backgrounds, bar lines, the word header and the playback plate.
  Use the audio position. Preserve unavailable and stale snapshot handling. Keep the UI outside sound timing.
- [x] 2.3 Keep F2 on terminal cells. Pad-parameter curves, the splash and audio recording keep the strip.
- [x] 2.4 Extend the offline catalogue for straight, triplet, off-grid, multiple-event, hidden-grid and safety states.
- [x] 2.5 Show A1 banks in the bank row: underline banks with visible onsets and light closed-bank pads at their 1/16 playback step.
  Test stopped and playing transport, the open bank, stale or missing audio position and hidden rows.

## 3. Help

- [x] 3.1 Show the effective view keys in the Pads help line when A1 exists. Explain the strip and bank-row marks in F1.

## 4. Verification and owner handoff

- [x] 4.1 Run the active-scenario audit with real named tests for every 98 scenario.
- [x] 4.2 Run `task check`, `task openspec:check` and the ordinary main-folder `task build`.
- [x] 4.3 Owner check of the view in the owner's terminal: the owner reported "все ок" for the 21:19 build on 2026-11-11.

## Earlier trials

The owner rejected a white/yellow cursor (low contrast), then found a white inverse plate visible but off center.
The agent selected the centered light plate after offline mock-ups, by owner delegation.
Physical-controller use and kitty-graphics composition remain unverified. The view adds no sound and no WAV.
