# 87a: Number pad copies without repeated suffixes

## Status

The owner approved this naming direction and specification preparation on 2026-11-06.
Change 87 is complete and merged into `master` at `93362d9`.
The owner approved the three naming choices below after that merge.
This change stays deferred until approval of activation and the implementation file plan.
The current step changes only these four draft documents, without code, commits or publication.

## Why

Each pad copy currently adds ` 2` to the displayed source name.
Copies of copies produce names such as `rec-20261103-194837 2 2 2`.
The owner cannot tell which numbers identify slices and which identify copies.

## What changes

- New copies use one numbered marker: `kick (copy 1)`, `kick (copy 2)`, `kick (copy 3)`.
- A copy of a copy replaces its final copy marker instead of adding another marker.
- A new name avoids occupied names across all banks of the current project or scratch session.
- A slice number stays in the base name: `break 2 (copy 1)`.
- Existing source names stay unchanged when the owner copies a pad.
- The next copy reuses the smallest free positive number, including numbers freed by deletion or undo.
- A final canonical ` (copy N)` marker has the same meaning in a manual source name.
- The target's current name reserves its number until replacement succeeds.

The owner approved these last three choices. `design.md` records their consequences and the remaining activation work.

## Out of scope

Do not clean old ` 2` suffixes automatically.
Do not rename sample files, change saved formats, add controls or change sound processing.
Do not redesign name clipping or implement this behavior inside change 87.
Unchanged names would be simpler, but they would not distinguish copies from the source.

## How to verify

These procedures apply after implementation, not to this deferred draft.
Use disposable work with no names that already contain `(copy N)`.

### Keyboard

1. On Pads, select a loaded pad named `kick`.
2. Press Ctrl+C.
3. Hit an empty target pad.
4. Verify `kick (copy 1)` on the selected pad.
5. Copy that pad to another empty target.
6. Verify `kick (copy 2)`, without a second marker.
7. Copy the original to an empty target in another bank.
8. Verify `kick (copy 3)`.
9. Save the work.
10. Reopen the project or scratch session.
11. Verify the same names.

### Controller

1. Repeat the copy and target steps with an existing learned copy control and controller pads.
2. Use the existing bank control to select a target in another bank.
3. Verify the same names and unchanged source pads.
4. Use the keyboard for save or confirmation if those actions have no controller binding.

### Sound and limits

Sound stays unchanged. No new WAV is necessary for this naming change.
Retain the automated copy-sound regression tests and the local `renders/83a-pad-copy.wav` render.
Existing narrow-cell clipping can hide the marker in the grid. Inspect the selected pad name too.
Physical-controller and listening results remain separate from automated results.
