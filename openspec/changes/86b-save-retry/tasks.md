# Tasks

## 1. Specify save retry

- [x] 1.1 Check the proposal, design and four spec deltas. Review the English text and commit the artifacts after `task check` passes.

## 2. Retain work and permit a retry

- [x] 2.1 Add named regression tests for write refusal, repeated requests, successful retry and later autosave. Use controlled time and isolated files.
- [x] 2.2 Separate read protection, pending changes and write failure. Return an explicit save result. Keep new coordination outside `main.rs`.
- [x] 2.3 Preserve the last saved bytes through safe temporary-file replacement. Refuse a removed project parent without recreating it. Keep scratch-folder creation.
- [x] 2.4 Test damaged and newer files, occupied temporary paths, the removed-folder case and paused exit saving. Keep the existing save scenarios covered.
- [x] 2.5 Run the project checks and commit this step with its tests after `task check` passes.

## 3. Show each save result

- [ ] 3.1 Route keyboard and learned raw MIDI save requests through production dispatch. Test matching success, refusal and read-protection feedback.
- [ ] 3.2 Add the persistent status indicator and screen tests at small and large supported sizes. Keep notices, help, controls and layout unchanged.
- [ ] 3.3 Test expired and replaced notices, successful retry and the next edit. Use controlled time, not sleeps.
- [ ] 3.4 Run the project checks and commit this step with its tests after `task check` passes.

## 4. Check and hand over

- [ ] 4.1 Review the diff and repeat the edge-case review. Record which cases pass and which remain open.
- [ ] 4.2 Run `task build`, `task check` and `task openspec:check`. Keep logs and report callback maxima separately from the exit result.
- [ ] 4.3 Add exact owner steps and verification results to `docs/LIVE-CHECKS.md`. State that sound processing stays unchanged and no new render is necessary.
- [ ] 4.4 Commit verification records after `task check` passes. Keep physical-controller and listening results open until the owner reports them.
- [ ] 4.5 Obtain owner acceptance before archive. Use the OpenSpec archive command and check scenario coverage before activating 86c.
