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

- [x] 3.1 Route keyboard and learned raw MIDI save requests through production dispatch. Test matching success, refusal and read-protection feedback.
- [x] 3.2 Add the persistent status indicator and screen tests at small and large supported sizes. Keep notices, help, controls and layout unchanged.
- [x] 3.3 Test expired and replaced notices, successful retry and the next edit. Use controlled time, not sleeps.
- [x] 3.4 Run the project checks and commit this step with its tests after `task check` passes.

## 3a. Learn saving from F12

- [x] 3a.1 Add the approved learning requirements and review the English text. Commit the amendment after `task check` passes.
- [x] 3a.2 Add `session.save` as the button row `save session` in GLOBAL. Keep existing keys, factory assignments and formats.
- [x] 3a.3 Test F12 navigation, pending confirmation and cancellation with raw note and CC buttons. Do not inject the save assignment.
- [x] 3a.4 Reload the saved configuration. Test saving on each input screen, write refusal, manual retry and resumed autosave.
- [x] 3a.5 Run the project checks and commit the row with its tests after `task check` passes.

## 4. Check and hand over

- [x] 4.1 Review the diff and repeat the edge-case review. Record which cases pass and which remain open.
- [x] 4.2 Run `task build`, `task check` and `task openspec:check`. Keep logs and report callback maxima separately from the exit result.
- [x] 4.3 Add exact owner steps and verification results to `docs/LIVE-CHECKS.md`. State that sound processing stays unchanged and no new render is necessary.
- [x] 4.4 Commit verification records after `task check` passes. Keep physical-controller and listening results open until the owner reports them.
- [x] 4.5 Obtain owner acceptance before archive. Use the OpenSpec archive command and check scenario coverage before activating 86c.
