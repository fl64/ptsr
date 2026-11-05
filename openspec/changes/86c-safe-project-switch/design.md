# Design

## Context

86b is archived. Its save operation returns `Result<(), String>` and retains pending work on refusal.
86c follows the agreed save-safety plan. 86d and 87 are not active.

Code observations at `cd8a472`:

- `main.rs:775` loads a target and applies empty state on read failure.
- `main.rs:804` creates a target before any source save.
- `main.rs:987` takes the picker and typed name before attempting a transition.
- `dispatch.rs:194` opens the picker through a logical action.
- Picker selection, confirmation and name input use keyboard events. MIDI actions do not navigate that picker.

## Goals / Non-Goals

Keep current work if a required save or target read fails.
Save pending work before a successful transition, including scratch work.
Keep existing startup protection, sound isolation, keys and formats.

Do not add a quit guard, controller-only picker controls, path migration, recovery or a dependency.

## Decisions

### Check an existing target before saving the source

Compare the target with the current folder first. For the same folder, close the picker without a read, save or state replacement.
Use folder identity, including a canonical path comparison when both folders exist.
For another existing target, read and check its session before the source save.
Add a strict `session.rs` loader for interactive opens. Keep the existing missing-file default only for startup.
A missing target file is a read refusal, not an empty project. Startup keeps its existing missing-file behavior.
Only after validation and any required save may the project, loader generations, audio epoch and last-project setting change.

Reject saving before the target check: a failed target read would clear the pending flag and could hide the original work state.
Reject an empty fallback on target read failure: it discards the visible current work.

### Save only pending source work

A clean source needs no write. This includes a clean scratch session with read protection.
If work is pending, call the 86b save operation even after an earlier write refusal.
A transition request is explicit and may retry that save. Failure blocks the transition.
Do not clear read protection to permit a transition. Successful source saving is silent.
A later target-creation failure keeps the source open with its work safely saved, not falsely marked pending.

### Keep picker ownership until success

Return a success flag from open and create. Consume the picker only on success.
Pass a clone of the typed name instead of removing it before the result.
On source refusal, keep selection, typed name, pads, chops, undo and the original pending timestamp.
On target read refusal, do not write the source, change its save flags or update the last-project setting.
A refusal notice may change. No new question or mode is necessary.

### Guard creation before filesystem changes

An existing project name uses the ordinary open path, including the same-folder check.
For a new name, save pending source work before creating the target folders or file.
If creation fails after that save, retain the source and picker. Partial empty target folders may remain.
Do not remove files to hide a creation refusal. No multiwriter guarantee enters this change.

### Test the existing input boundary

Run shared edits and picker entry through keyboard and raw MIDI using the production profile and dispatcher.
Keep selection, confirmation and typed names on their existing keyboard path in both runs.
Do not add a test-only MIDI confirmation route or claim a controller-only picker.
Update fixtures that used interactive project opening to simulate startup read protection.
Existing project-isolation renders continue to test actual sound and late-loader boundaries.

## Edge cases

| Case | Required result | Status | Check |
| --- | --- | --- | --- |
| Edit then open before autosave | Save the source before replacement | Open, support now | Recorded keyboard and MIDI edit sequences |
| Dirty scratch then open or create | Save scratch, not only named projects | Open, support now | Saved-byte reload |
| Source write refusal, repeated Enter | Keep work, undo, selection and typed name | Open, support now | Occupied temporary path and repeated requests |
| Protected source with pending work | Refuse transition without changing either session | Open, preserve protection | Protected source fixture |
| Clean protected scratch | Leave without writing its file | Open, support now | Byte comparison |
| Same folder with pending work or an occupied temporary path | Close without a save or reload | Open, support now | State, epoch and file comparisons, including an alias |
| Damaged, newer, unreadable or missing target | Keep source and last-project setting | Open, support now | Target faults before source saving |
| New target creation fails after source saving | Keep source and typed name. Retain the saved source work | Open, support now | Occupied target path |
| Late loader reply after a successful switch | Preserve existing project sound isolation | Preserve existing behavior | Existing loader and render tests |
| Power loss or concurrent project writers | No new guarantee | Outside this change | No acceptance claim |

The axes are source type, pending work, protection, target identity, failure order, repeated requests and input route.
The UI executes transitions synchronously. Background sample replies retain the existing generation checks.
Hardware and listening remain unverified. Callback maximum-budget acceptance stays open.

## Open questions

None within the agreed scope. Controller-only picker navigation remains outside 86c.
The owner waived the manual passes of 86b, not proof of any hardware or listening result for later changes.
