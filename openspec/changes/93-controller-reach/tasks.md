# Tasks

## 1. Evidence and specification

- [x] 1.1 Read current action metadata, keyboard and MIDI routes, panel cells, modal handlers, view states and their tests. Count the actions and record route gaps by screen and mode.
- [x] 1.2 Record the contract for text entry, SMC-PAD Exit, press counts, page selection, pickers, Settings and Help. Add edge cases and defaults before code.
- [x] 1.3 Update the outdated splash and `ActionInfo` suggestions in `docs/plans/SMC-PAD-INTEGRATION.md`. Keep the accepted sound-first splash behavior. Update `docs/VISION.md` only if a chosen path changes its controller goal.
- [x] 1.4 Run `task openspec:check` and STE lint. Commit the specification step.

## 2. Complete route inventory

- [x] 2.1 Add `docs/CONTROLLER-PATHS.md`. Give every story in `docs/USER-STORIES.md` a controller route, first-entry count and repeated count, or its explicit exception.
- [x] 2.2 Add text-entry metadata for the two listed actions without adding `app.quit` to MIDI learn. Test the allowed text exceptions and reject an unlisted exception.
- [x] 2.3 Add a test that uses the shipped keyboard map, SMC-PAD profile and panel map through production dispatch. Cover each screen and mode in both directions. Report missing and over-budget routes with their context. Test visible mode badges.
  The inventory checks every shipped mapping and press limit. Raw MIDI checks cover 225 panel cells, all 16 pad hits, direct transport buttons and global knobs in eight views. Result checks cover contextual knobs in Pads, Chop, Files, Pattern overview and Settings, plus the open A1 controls. Badge tests cover visible modes.
- [x] 2.4 Add the missing cell labels and action IDs to this design and delta before their code. Check and commit these OpenSpec edits separately.
- [x] 2.5 Record the owner-approved `Alt+U` route in the keyboard delta and design. Keep Files and Chop Space previews. Update the affected plan and owner checks before code. Run `task openspec:check` and commit these text edits.
- [x] 2.6 Record `Alt+Left` and `Alt+Right` for silent pad selection in the delta and design. Keep plain value arrows on Pads and Alt-arrow slice bounds on Chop. Update the plan and owner checks, then check and commit the text before code.
- [x] 2.7 Bind the silent selection keys on Pads and Pads edit. Compare their results with raw SMC-PAD LEFT and RIGHT, including row edges, Gate release, repeat, owner bindings and overlays. Keep other screens unchanged.
- [x] 2.8 Record the owner-approved exact Chop threshold keys in the delta, design, proposal, plan and owner checks. Check and commit these texts before code.
- [x] 2.9 Add the threshold actions and keys. Compare keyboard and raw encoder 3 in Chop and Chop edit, including bounds, Undo, linked pads, owner bindings, overlays and legacy terminals. Complete the named scenarios before the code commit.
- [x] 2.10 Load the shipped A1 encoder roles as a separate MIDI context. Add the group to new and already seeded profiles without changing owner bindings. Show the roles in the Controller tab. Test keyboard and raw MIDI parity on every underlying screen, close, input owners, other devices and held Gate releases.

## 3. Picker and Settings routes

- [x] 3.1 Route encoder 1, P7, P8 and Back in the project and Paths folder pickers. Preserve question priority, failed-save selection, another device's route and held Gate releases. Test keyboard and raw SMC-PAD runs.
- [x] 3.2 Route Settings encoder 2 through the current value action and bounds. Keep it from changing values while a picker is open. Test both input paths and owner binding overrides.
- [x] 3.3 Show the current controller picker controls in both terminal sizes. Check their screen catalogue entries. Keep this code in the implementation step until all active scenarios pass.

## 4. Panel pages and Help

- [x] 4.1 Make encoder 6 select later context pages only while the SMC panel is visible. Keep More, Next, Main, reserved bottom cells, first-hit entry and LEFT/RIGHT item selection. Test raw MIDI, keyboard parity and two controllers.
- [x] 4.2 Make Help navigable from its panel cell. Test P7 through P10, Back, question priority and a held Gate. Keep F1 and keyboard Help navigation.
- [x] 4.3 Fill remaining non-text action paths shown by the inventory. Keep each path within the specified press limit. Add named keyboard and raw MIDI scenarios.
- [x] 4.4 Add Exit to a context page. Make the first hit open a question, P8 run the existing quit path once, and P7 or Back cancel. Keep Ctrl+Q and MIDI learn behavior. Test a pending save or recording.

- [x] 4.5 Record the measured PAD BANK pulse, optional MIDI setting and panel entry rules in OpenSpec.
- [x] 4.6 Document both PAD BANK entry paths in the controller paths, user stories, owner checks, plan and vision. Check the new text before code.
- [ ] 4.7 Add the MIDI setting and read-only status detection. Use the existing panel path. Test saving, pulse detection, source, overlays, input order, fallback and Gate release. Check the new Settings row in both terminal sizes.

## 5. Owner documentation and final verification

- [x] 5.1 Update `docs/USER-STORIES.md` and `docs/LIVE-CHECKS.md` with exact keyboard and SMC-PAD steps. Report every proposal item and its screen-catalogue state before the owner check.
- [x] 5.2 Complete the implementation step after tests name all active scenarios. Run format, changed-crate Clippy, `task test:changed`, `task screens`, `task test:fast`, `task specs:check`, `task names`, STE lint and `git diff --check`. Report each required failure, including one resolved later. Commit the code only after these checks pass. This change has no new render or audio-thread benchmark requirement.
- [ ] 5.3 Publish one PR to `master`. Wait for Change names, Specs and Rust CI to pass. Keep unreported physical and listening checks open until the owner reports them.
