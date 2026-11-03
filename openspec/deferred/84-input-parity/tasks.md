## 0. Activate and confirm scope

- [ ] 0.1 Merge document preparation, finish keymap cleanup 83a, then activate this
  draft on its own branch. Recheck the Chop routes and test helpers against that base.
  Record any changed assumptions before code.

## 1. Paired input driver

- [ ] 1.1 Add the test driver for key events, raw MIDI bytes and explicit frames. Reuse
  the production dispatch and profile initialization. Keep extraction separate from
  behavior changes. Run `task check` and commit.
- [ ] 1.2 Add paired baseline tests for Pads, edit, marking, file in hand, swap,
  question and learn. Keep X-Session default tests unchanged. Record differences in
  design.md; stop for a scope decision instead of hiding a new fix. Run `task check`
  and commit.

## 2. Chop parity

- [ ] 2.1 Add failing regression tests for all six delta scenarios through raw bytes
  and App. In the same step, preserve velocity in PadHit and route Chop hits through
  the keyboard Chop handler. Check banks, selection, plans and no duplicate audio.
  Run `task check` and commit the fix with its tests.
- [ ] 2.2 Check velocity, note-off forms, original held-pad release after bank and
  screen changes, queue acceptance and project epoch filtering. Render
  `renders/84-input-parity.wav` through the implemented route; check finite samples,
  no clipping, silence, levels and nearest-block timing. Run `task check` and commit.

## 3. Documentation and acceptance

- [ ] 3.1 Update user stories and live checks with separate keyboard and controller
  passes. Validate the change, run `task check`, build the release and commit.
- [ ] 3.2 Ask the owner to check Chop listening and assignment from both inputs and
  listen to `renders/84-input-parity.wav`. Record the result, not an assumed pass.
- [ ] 3.3 Archive with `openspec archive` only after acceptance and scenario coverage
  in the main specs. Validate specs, run `task check` and commit the archive.
