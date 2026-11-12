# Tasks

## 1. Agree on the contract

- [x] 1.1 Compare the deferred editor draft with the accepted F5 overview and write a new OpenSpec proposal.
- [x] 1.2 Record the default entry key, first edit scope and event detail in the design. Keep the open questions for owner review.
- [x] 1.3 Fix the old draft link and keep the active specifications as the only implementation contract.

## 2. Input and selection

- [x] 2.1 Add the correction mode to the shared input snapshot. Test keyboard and raw MIDI silent routes and original Gate releases.
- [x] 2.2 Add the F7 entry action, effective step and event actions, bank and view navigation, and blocked-mode refusals.
- [x] 2.3 Select visible raw-row identities in a cell. Test hidden rows, collisions, stale identity, project changes and input order.

## 3. Correction and sound

- [x] 3.1 Remove one checked raw row through phase-preserving publication. Test pressure, playback boundaries and held-note tails.
- [x] 3.2 Add one-event Undo and safe save. Test common and pattern Undo, refusal, reload and unchanged settings.
- [x] 3.3 Render the correction and Undo with generated sounds. Check timing, silence, level, clipping and clicks.

## 4. Screen and handoff

- [x] 4.1 Draw an independent edit cursor, event index and count, mode badge, notices and effective help at both supported sizes.
- [x] 4.2 Add offline catalogue states and named tests for every delta scenario.
- [x] 4.3 Run `task check`, `task openspec:check` and `task build`. Keep callback maximum limits visible.
- [x] 4.4 Give the owner exact keyboard, controller, screen and listening steps with the new render path.
