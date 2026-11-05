# Portable chop variants

## Why

The project already stores internal pad and current-chop paths relative to its folder.
The path helpers omit `Session::kept`, which holds older pad-linked chops.
After a folder move, those keys can still name the old folder.
Opening an older slice can then rebuild only its pad bounds and lose cuts with no assigned pad.

86b and 86c are accepted and archived. 86d is the next agreed save-safety change.

## What Changes

- Apply existing stored and resolved path rules to every saved chop variant.
- Keep internal paths relative and external paths absolute without copying files.
- Preserve complete cuts, regions, cut IDs, epochs, next IDs, pad links and sound settings.
- Test relocation through production saving, project loading and keyboard/raw-MIDI input.
- Render restored slices for automated tests and owner listening.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `projects`: include every saved chop variant in project path conversion and relocation behavior.
- `chopping`: preserve older variants and unassigned cuts after relocation, with isolated edits and matching sound.

## Impact

Code: the two project path helpers in `crates/app/src/main.rs`.
Tests: recorded input, project load/save and offline rendering in `crates/app/src/input_tests.rs`.
Records: this change and `docs/LIVE-CHECKS.md`.
Render: `renders/86d-portable-chops.wav`.

No saved-format change, sample copy, dependency, key change or sound-processing change.
Do not guess paths for projects that already moved with stale absolute keys.
Keep existing fallback for an old project that has no saved older variant.
The callback-budget investigation is separate. No result here closes its timing limits.

## How to verify

### Keyboard

1. Use a disposable project with an internal sample and at least two pad-linked chops of it.
2. Keep an extra cut with no assigned pad in an older chop.
3. Save with Ctrl+S. Close the program. Move the project folder within the projects root.
4. Open the moved project with Ctrl+P. Select one pad from each chop and press F3.
5. Verify all cuts, including the extra cut. Move a bound in one chop.
6. Verify that only its linked pads follow. Save and reopen the project again.

### Controller

Repeat the shared edits, saving and screen entry with learned controls and raw MIDI in automated tests.
For owner verification, use learned controls for those same actions.
Keep picker selection, Enter and typed names on their existing keyboard path.
Do not infer physical-controller acceptance from recorded input.

### Listening

Listen to `renders/86d-portable-chops.wav`. Compare each slice before and after relocation.
Automatic tests compare matching samples. They verify finite values, no clipping, expected silence and hit timing within one sample.
