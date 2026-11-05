# Proposal

## Why

Opening or creating a project can replace pending work before autosave runs.
A target read refusal currently replaces the current session with empty state.
The owner needs project changes that keep the current work when saving or reading fails.

## What Changes

- Save pending project or scratch work before a transition.
- On a save refusal, retain pads, chops, undo, pending work and the picker selection or typed name.
- Verify an existing target before saving or replacing the current session.
- On a target read refusal, retain the current project and last-project setting.
- Close the picker without reloading when the owner selects the current project.
- Keep startup read protection and existing input paths.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `projects`: Guard open and create, preserve the picker on refusal, and avoid reloading the current project.
- `pad-assignment`: Save pending scratch or project work before replacement.
- `screen-help`: Explain transition refusals without a new confirmation mode.

## Impact

Change the existing project methods and picker in `crates/app/src/main.rs`.
Add strict interactive loading in `session.rs`, while startup keeps its existing missing-file default.
Add recorded-input coverage in `input_tests.rs` and adapt the read-protection fixture in `saving_tests.rs`.
Keep keys, profiles, saved formats, sound processing and the existing project-isolation renders unchanged.
No dependency is necessary. Changes 86b–86d share one branch and PR by owner agreement.
Only 86c is active. Push, PR publication and merge require separate authorization.

## Out of scope

Do not add a quit guard, controller-only picker navigation, automatic recovery, path migration or change 87.
Do not claim hardware, listening or maximum-budget acceptance from automated test results.

## How to verify

1. In a disposable project, change a pad and open another project before autosave.
2. Return to the first project. Verify that the change remains.
3. Repeat from scratch and when creating a new project.
4. Occupy the source session's `.toml.tmp` path with a directory. Edit and try each transition.
5. Verify the retained pads, chops, undo, selection and typed name after each refusal.
6. Remove only that test directory. Repeat the transition and verify the saved source.
7. Select the current project with pending work. Verify that the picker closes without reloading or saving.
8. Select a damaged target. Verify the refusal and the unchanged current project and last-project setting.

Keyboard and controller results stay separate. For the controller pass, use existing learned actions for editing and `project.pick`.
Select in the picker with the keyboard and press Enter. Controller navigation is not implemented.
No new WAV is necessary because the sound algorithm stays unchanged.
Listen to `renders/79-project-switch.wav` and `renders/11-switch-nothing-leaks.wav` as existing regression references.
