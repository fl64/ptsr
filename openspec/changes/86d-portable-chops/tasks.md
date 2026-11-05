# Tasks

## 1. Specify portable variants

- [x] 1.1 Check the proposal, design and two deltas. Check English text and OpenSpec, then commit after `task check` passes.

## 2. Correct the path boundary

- [ ] 2.1 Add project fixtures with several older variants, unassigned cuts and internal/external samples. Test production load/save and keyboard/raw-MIDI routes.
- [ ] 2.2 Apply existing stored/resolved path conversion to every `kept` key without changing variant states or formats.
- [ ] 2.3 Test the second save/open after relocation, isolated bound edits, absent `kept` and stale absolute keys.
- [ ] 2.4 Render restored slices before and after relocation. Check equality, finite samples, clipping, silence and timing, and write the owner WAV.
- [ ] 2.5 Review the diff and run `task check`. Commit code and tests only after it passes.

## 3. Check and hand over

- [ ] 3.1 Repeat the edge-case review and check named coverage for every delta scenario.
- [ ] 3.2 Run `task build`, `task check` and `task openspec:check`. Report callback maxima separately from exit results.
- [ ] 3.3 Record exact keyboard/controller steps, render timing and unverified results in `docs/LIVE-CHECKS.md`.
- [ ] 3.4 Commit the verification records after `task check` passes.
- [ ] 3.5 Obtain owner acceptance before archive. Do not activate 87 or publish without separate authorization.
