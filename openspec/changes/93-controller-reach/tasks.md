# Tasks

## 1. Evidence and specification

- [ ] 1.1 Read current action metadata, keyboard and MIDI routes, panel cells, modal handlers, view states and their tests. Count the actions and record route gaps by screen and mode.
- [x] 1.2 Record the contract for text entry, keyboard-only quit, press counts, page selection, pickers, Settings and Help in the proposal, design and spec delta. Record edge cases and defaults before code.
- [x] 1.3 Update the outdated splash and `ActionInfo` suggestions in `docs/plans/SMC-PAD-INTEGRATION.md`. Keep the accepted sound-first splash behavior. Update `docs/VISION.md` only if a chosen path changes its controller goal.
- [x] 1.4 Run `task openspec:check` and STE lint. Commit the specification step.

## 2. Complete route inventory

- [ ] 2.1 Add `docs/CONTROLLER-PATHS.md`. Give every story in `docs/USER-STORIES.md` a controller route, first-entry count and repeated count, or its explicit exception.
- [ ] 2.2 Add text-entry metadata for the two listed actions without adding `app.quit` to MIDI learn. Test the allowed exceptions and reject an unlisted exception.
- [ ] 2.3 Add a test that uses the shipped keyboard map, SMC-PAD profile and panel map through production dispatch. Cover each screen and mode in both directions. Report missing and over-budget routes with their context. Test visible mode badges.
- [ ] 2.4 Add the missing cell labels and action IDs to this design and delta before their code. Run required changed-crate checks and commit a working inventory step.

## 3. Picker and Settings routes

- [ ] 3.1 Route encoder 1, P7, P8 and Back in the project and Paths folder pickers. Preserve question priority, failed-save selection, another device's route and held Gate releases. Test keyboard and raw SMC-PAD runs.
- [ ] 3.2 Route Settings encoder 2 through the current value action and bounds. Keep it from changing values while a picker is open. Test both input paths and owner binding overrides.
- [ ] 3.3 Show the current controller picker controls in both terminal sizes. Run format, changed-crate Clippy, `task test:changed` and the screen catalogue comparison. Commit this working step.

## 4. Panel pages and Help

- [ ] 4.1 Make encoder 6 select later context pages only while the SMC panel is visible. Keep More, Next, Main, reserved bottom cells, first-hit entry and LEFT/RIGHT item selection. Test raw MIDI, keyboard parity and two controllers.
- [ ] 4.2 Make Help navigable from its panel cell. Test P7 through P10, Back, question priority and a held Gate. Keep F1 and keyboard Help navigation.
- [ ] 4.3 Fill remaining non-text action paths shown by the inventory. Keep each path within the specified press limit. Add named keyboard and raw MIDI scenarios. Run required checks and commit this working step.

## 5. Owner documentation and final verification

- [ ] 5.1 Update `docs/USER-STORIES.md` and `docs/LIVE-CHECKS.md` with exact keyboard and SMC-PAD steps. Report every proposal item and its screen-catalogue state before the owner check.
- [ ] 5.2 Run format, changed-crate Clippy, `task test:changed`, `task screens`, `task test:fast`, `task specs:check`, `task names`, STE lint and `git diff --check`. Report each required failure, including one resolved later. This change has no new render or audio-thread benchmark requirement.
- [ ] 5.3 Publish one PR to `master`. Wait for Change names, Specs and Rust CI to pass. Keep unreported physical and listening checks open until the owner reports them.
