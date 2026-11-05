# Proposal

## Why

A failed save clears the unsaved state and blocks later saves without clear feedback.
The owner needs to retain the work and retry after the write problem ends.
This protects the sample-to-project part of the v1 core loop.

## What Changes

- Retain unsaved changes until the file replacement succeeds.
- Separate a write failure from protection after a read error.
- Report success or a refusal reason for every manual save.
- Add `save session` to GLOBAL on the F12 MIDI tab. Use the existing learning and confirmation steps.
- Pause automatic retries after a write failure. Resume autosave after a successful manual save.
- Show a persistent `Unsaved` or `Save failed` status without a new input mode.
- Keep the last saved file intact on failure. Do not recreate a removed project folder.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pad-assignment`: Retain changes after failed saves and permit manual retries.
- `projects`: Protect unreadable files and refuse writes into removed project folders.
- `screen-help`: Report each manual save result and let the owner learn its controller button from F12.
- `screen-layout`: Show the persistent save state on every screen.

## Impact

The affected areas are session saving, save dispatch, status drawing and their tests.
Keys, profiles, saved formats and sound processing stay unchanged. No dependency is necessary.
The owner approved sequential work on 86b, 86c and 86d in one branch and one PR.
Only 86b is active. Push, PR publication and merge need separate authorization.

## Out of scope

Do not change project switching, sample path rules, sound processing or the audio callback.
Do not add save history, crash recovery, a quit question or change 87.
Do not claim hardware, listening or callback-budget acceptance from offline test results.

## How to verify

1. Create a disposable project. Load a sample onto a pad. Save with Ctrl+S.
2. Change its volume. In another terminal, run `chmod u-w /path/to/project` for that disposable project folder.
3. Press Ctrl+S twice. Verify a refusal reason each time and the persistent `Save failed` status.
4. Run `chmod u+w /path/to/project`. Press Ctrl+S. Verify `Session saved` and no save-state warning.
5. Change the volume again. Let autosave run. Reopen the project and verify the saved volume.
6. Open F12 and select the MIDI tab. Open GLOBAL and select `save session`.
7. Press Enter. Press an unused controller button. Press Enter to save its assignment.
8. Restart the program. Repeat the save actions with that button. Keep keyboard and controller results separate.

Automated tests control time and inject write refusals without a full disk or hardware.
They compare the last saved bytes and verify read protection and a removed project folder.
Permission steps require a normal owner account, not root. Automated tests do not depend on permission bits.
This change adds no sound render. Sound processing stays unchanged. Listening remains an owner task.
