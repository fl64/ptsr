# Tasks

## 1. Specify safe transitions

- [x] 1.1 Review the proposal, design and three deltas. Check the English text and commit after `task check` passes.

## 2. Guard opening and creation

- [x] 2.1 Add recorded-input tests for pending source saving, scratch, save refusal and retained picker state. Use production keyboard and raw MIDI routes.
- [x] 2.2 Check an existing target before saving the source. Reject missing files on interactive open without changing startup loading.
- [x] 2.3 Save pending source work before open or create. Preserve picker selection and typed name on refusal.
- [x] 2.4 Avoid reloading the current folder, including an alias. Check state, loader generations, audio epoch and saved bytes.
- [x] 2.5 Test read protection, target read faults, creation refusal and retries. Adapt startup and same-folder fixtures without weakening their intended checks.
- [x] 2.6 Run `task check`, inspect the diff and commit code with its tests.

## 3. Check and hand over

- [x] 3.1 Repeat the edge-case review. Check named coverage of all delta scenarios and the existing project-isolation renders.
- [x] 3.2 Run `task build`, `task check` and `task openspec:check`. Report benchmark maxima separately from exit results.
- [x] 3.3 Record exact owner steps and results in `docs/LIVE-CHECKS.md`. Keep keyboard, controller and listening evidence separate.
- [x] 3.4 Commit the verification records after `task check` passes.
- [x] 3.5 Obtain owner acceptance before archive and activation of 86d.
