## 0. Activate and confirm scope

- [x] 0.1 Activate after the 83a archive (`2b084d1`). Keep the existing branch
  `plan/controller-first-patterns` by owner request; no preparation merge is needed.
  Recheck the Chop routes and test helpers against that base. Record the branch
  exception, baseline and changed assumptions before code. Validate strictly,
  run `task check` and commit these artifacts separately.

## 1. Paired input driver

- [x] 1.0 Extract shared MIDI state publication unchanged for production and tests.
  Cover the published fields without changing their rules. Run `task check` and commit.
- [x] 1.1 Add the test driver for key events, raw MIDI bytes and explicit frames. Reuse
  the production dispatch and profile initialization. Keep extraction separate from
  behavior changes. Run `task check` and commit.
- [x] 1.2 Add paired baseline tests for Pads, edit, marking, file in hand, swap,
  question and learn. Keep X-Session default tests unchanged. Record differences in
  design.md; stop for a scope decision instead of hiding a new fix. Run `task check`
  and commit.

- [x] 1.3 Add failing raw-MIDI regressions for silent copy targets, replacement and
  original held-pad release. Publish the copy hand and keep the visible target in sync
  with the keyboard. Test notes and learned pad buttons across banks, empty and occupied
  targets, cancel and confirm. Run `task check` and commit this repair separately.

- [x] 1.4 Repair the three task 1.2 differences approved by the owner: select the
  same file and swap target, record the same swap undo step, and route new pad hits
  under a question to the silent UI guard. Keep confirmation commands and original
  held-pad releases. Cover the four new delta scenarios, cancellation, loading refusal
  and return to direct performance. Run `task check` and commit with the paired tests.

## 2. Chop parity

- [x] 2.1 Add failing regression tests for all six Chop delta scenarios through raw bytes
  and App. In the same step, preserve velocity in PadHit and route Chop hits through
  the keyboard Chop handler. Check banks, selection, plans and no duplicate audio.
  Include the owner-approved linked gate release repair and five additional release
  scenarios: audio delivery, original target, silent-hit exclusion, direct-route/epoch
  isolation and retention of an accepted hold when a queue-full retrigger is rejected.
  Check note-off forms, learned CC0, disconnect, guards and offline gate silence.
  Add the separately approved held-keyboard release through screen learn with paired
  input coverage: original pad after bank/screen changes, unchanged learn pick/pending
  bindings, and no release from new silent keys, including keys of a held MIDI pad.
  Run `task check` and commit the artifacts separately, then the fix with its tests.
- [x] 2.2 Check velocity, note-off forms, original held-pad release after bank and
  screen changes, queue acceptance and project epoch filtering. Render
  `renders/84-input-parity.wav` through the implemented route; check finite samples,
  no clipping, silence, levels and nearest-block timing. Run `task check` and commit.

- [x] 2.3 Add the owner-approved green playback line for linked pad hits in the open
  Chop listen and slice editor. First add failing tests for all six chopping delta
  scenarios, using keyboard, raw notes and learned CC through the real offline queues.
  Report real pad positions without an extra voice. Keep display position separate
  from preview transport/edit position; do not change Space, M, cursor control,
  hit-driven selection or sound. Cover pitch, retrigger, stop/release, matching source and
  project, queue rejection, question/learn guards and both waveform modes. Review the
  documented concurrent-voice default before code. Include the later owner-approved
  fixed-width LISTEN badge: dim at rest, blue with the existing play icon only during
  matching playback, including outside the visible wave. Preserve the Space header
  scenario and test the three new badge scenarios. Keep the assign badge unchanged.
  Commit this scope extension separately from code. Update user stories and live checks,
  validate, run task check, rebuild the release and commit the fix separately.

## 3. Documentation and acceptance

- [x] 3.1 Update user stories and live checks with separate keyboard and controller
  passes. Validate the change, run `task check`, build the release and commit.
- [ ] 3.2 Ask the owner to check Chop listening and assignment from both inputs and
  listen to `renders/84-input-parity.wav`. Include the pad playback line after 2.3.
  Record the result, not an assumed pass. The owner has no controller at present;
  keep its physical pass unverified, not passed or waived by automated MIDI tests.
- [ ] 3.3 Archive with `openspec archive` only after acceptance and scenario coverage
  in the main specs. Validate specs, run `task check` and commit the archive.
