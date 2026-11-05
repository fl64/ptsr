# Design

## Context

The active change is 86b. Changes 86c and 86d follow it in the same branch by owner agreement.
The main specifications and `docs/plans/README.md` define the scope.

The inspected code establishes these facts:

- `crates/app/src/main.rs:2182` clears `session_dirty` before checking protection or writing.
- `crates/app/src/main.rs:2263` treats a write error as permanent read protection.
- `crates/app/src/dispatch.rs:160` reports success but relies on the save routine to report refusals.
- `crates/app/src/main.rs:4653` calls autosave after one second of pending changes.
- `crates/app/src/session.rs:214` creates parent folders and replaces the destination through a temporary file.

These are code observations, not reproduced owner failures. Line numbers refer to the baseline `b3dec98`.

## Goals / Non-Goals

**Goals:** Retain unsaved work, permit a manual retry, explain each refusal and preserve the last saved file.
Use the same action and result for keyboard and raw MIDI.
Make the save state visible without hiding normal notices or changing screen geometry.

**Non-Goals:** No change to project transitions, sample paths, sound, keys, profiles or saved formats.
No automatic recovery, save history, quit question, new dependency or callback correction.

## Decisions

### Separate protection, pending changes and write failure

Keep read protection separate from retryable write failure.
A failed write must not clear the pending-change timestamp.
A successful replacement clears pending changes and write failure.
The save operation returns an explicit result for the caller.
This result will also support transition guards in 86c without activating that change now.

Reject one permanent failure flag: it prevents recovery after a temporary refusal.
Reject constant automatic retries: the approved plan pauses them until a manual save succeeds.

### Keep autosave scheduling testable

Use an explicit current-time argument for the autosave decision.
Keep the existing idle threshold. Do not use sleeps to test it.
After a write failure, automatic saves, including exit saving, do not retry.
A manual request may retry. After success, the next edit schedules autosave normally.
Read protection always refuses the write, including a manual request.

### Preserve the destination through replacement

Serialize first, write a sibling temporary file, then replace the destination.
An occupied temporary path must not allow truncation of the last saved file through a link.
Remove only a temporary file that this attempt created.
Do not remove or overwrite another file to make a failed save pass.
Project saving requires an existing parent folder and never creates it.
Scratch saving retains its existing ability to create its data folder.
No power-loss durability or simultaneous-writer guarantee enters this change.

### Show feedback in separate places

Manual success keeps `Session saved` for five seconds.
Manual refusal shows a current reason for five seconds, including repeated requests under read protection.
A successful autosave stays silent.
The shared status line shows `Unsaved` for pending work or `Save failed` after a write refusal.
`Save failed` takes precedence over `Unsaved`. No warning appears for a clean, writable session.
Normal notices, recording progress and modal controls retain their places and behavior.
Read protection explains that the file cannot be saved. A later write request cannot remove that protection.

### Learn saving through the existing MIDI list

The owner approved `save session` in GLOBAL on the F12 MIDI tab.
The list uses `input::ACTIONS`, which has no `session.save` entry at the baseline.
The action parser and learned configuration already accept that name.
Add one button action to that list. Keep its name `session.save`, its label `save session` and its group GLOBAL.
Use the existing pending-control confirmation, configuration writer and production dispatch.
Test note and CC buttons through F12, confirmation, restart and a failed save followed by a successful retry.
Do not inject the save assignment directly in those tests.

Reject a separate learning path: the existing list already supports global button actions.
Keys, factory assignments and saved formats stay unchanged.

### Keep the main module small

Place new save coordination and its tests in dedicated modules.
Limit changes to `main.rs` to state, initialization, call sites and removal of relocated save methods.
Do not move unrelated project or input logic as part of this step.

## Edge cases

Statuses describe planning coverage, not passing tests.

| Case | Required result | Status | Check |
| --- | --- | --- | --- |
| Autosave fails, then another edit occurs | Keep both edits and pause automatic retries | Open, support now | Controlled-time test and saved-byte comparison |
| Two manual retries fail | Report each refusal and retain work | Open, support now | Keyboard and production MIDI dispatch |
| Manual retry succeeds, then another edit occurs | Clear failure and resume ordinary autosave | Open, support now | Controlled-time test and reload |
| Startup reads damaged or newer data | Refuse all saves without replacing that file | Open, preserve behavior | Compare bytes before and after each request |
| Project folder disappears before or during save | Refuse without recreating its directory | Open, support now | Isolated filesystem fixture |
| Temporary path is occupied or linked | Refuse safely and keep saved bytes | Open, support now | Filesystem test with an occupied path or link |
| Another notice replaces the failure flash | Keep `Save failed` in the status line | Open, support now | Draw each screen at supported sizes |
| Program exits while a failed save remains | Do not retry automatically or claim success | Open, support now | Exit-save decision test |
| Learning receives a button before confirmation | Show the pending control without changing its saved assignment | Open, support now | Production F12 route with note and CC input |
| Learning ends with Esc | Keep the previous assignments and saved configuration | Open, preserve behavior | Cancel and compare configuration bytes |
| Save binding stored, then restart | Retain GLOBAL saving and the same retry feedback | Open, support now | Reload configuration into the production MIDI route |
| Two processes save the same project, or power fails | No new guarantee | Outside this change | No acceptance claim |

The inspected axes are source type, read protection, pending work, failure timing, retry order, input path and screen size.
Filesystem ownership and production input routing are part of the tests.
Hardware and power-loss behavior remain unverified.

## Risks / Trade-offs

- A save refusal still leaves work only in memory. The persistent status warns the owner, but this change adds no quit guard.
- A removed project directory must remain absent even if it disappears after an initial existence check.
- Permission tests can pass incorrectly under a privileged account. Use a controlled refusal fixture instead of permission bits alone.
- A successful benchmark command does not close the existing callback maxima or slow-core issue.
- New state must initialize in startup and test construction. Existing save, removed-folder and status scenarios must remain covered.

## Open questions

None for the save-learning amendment. The owner selected the F12 GLOBAL row and approved its implementation.
Keyboard, physical-controller and listening acceptance remain separate from implementation approval.
