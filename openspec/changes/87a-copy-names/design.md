# Design: Pad copy names

## Status and evidence

This is the active naming change on the separate `87a-copy-names` branch.
The preparation base was `origin/master` at `a4789f4`.
Change 87 is complete and merged into `master` at `93362d9`.
The implementation base includes that merge without rewriting the preparation history.
The owner approved the three naming choices below, activation, the implementation file plan and local signed-off commits.
Publication, archive and merge still need separate permission.
Change 87 remains separate. Its implementation and acceptance are not new work in 87a.

Confirmed baseline:

- `openspec/specs/pad-assignment/spec.md`, requirement `Copy a pad`, requires the displayed source name plus ` 2`.
- `crates/app/src/main.rs:2328`, `place_copy`, appends that suffix when it places a copy.
- `crates/app/src/main.rs:2348`, `copy_name`, prepares the name for the replacement question.
- `crates/app/src/main.rs:5544`, `pad_name`, selects a custom name or the file stem with its slice number.
- Existing copy tests cover the source snapshot, replacement, cancellation, bank changes, undo, saved names and matching sound.

These source locations describe the preparation base, not the later change-87 layout.

## Owner decisions

- Use a visible copy marker with one number: `(copy 1)`, `(copy 2)`, `(copy 3)`.
- A copy of a copy receives another available number without a repeated marker.
- Scan occupied names across every bank in the current project.
- Preserve the slice number in the base name.
- Do not clean existing ` 2` chains automatically. A number can belong to a slice or a manual name.
- Prepare a separate deferred change. Implement only after change 87 is complete.
- Reuse the smallest free positive copy number after clearing or undo. Numbers are names, not a copy history.
- Treat a canonical final ` (copy N)` in a manual name as the marker for its next copy.
  Keep the manual source name unchanged.
- Include the occupied target's current name when selecting a number, until replacement succeeds.
  Replacing `kick (copy 1)` with a copy of `kick` therefore selects `kick (copy 2)` if available.

## Naming rules

The three choices above are owner decisions, not unresolved defaults.
The review against `93362d9` found no change to the main copy requirements.
The existing question compares both the current target name and the proposed copy name before replacement.

### Base name and marker

Use the full displayed source name from the pickup snapshot, before screen clipping.
If that name ends in a canonical ` (copy N)` marker, remove that final marker to obtain the base name.
`N` is a positive ASCII decimal number without leading zeroes. The marker is case-sensitive and uses the shown spaces.
Keep all other text exactly, including Unicode, slice numbers and old ` 2` chains.
An existing marker need not fit a machine integer to identify the base name.

Examples:

| Source name | Base name | First candidate |
| --- | --- | --- |
| `kick` | `kick` | `kick (copy 1)` |
| `break 2` | `break 2` | `break 2 (copy 1)` |
| `kick (copy 8)` | `kick` | `kick (copy 1)` |
| `rec 2 2 2` | `rec 2 2 2` | `rec 2 2 2 (copy 1)` |
| `kick (copy 01)` | `kick (copy 01)` | `kick (copy 01) (copy 1)` |

The first candidate is not necessarily available. The source itself occupies its current full name.
Only the final canonical marker has this meaning. Earlier text is part of the base name.

### Available number

Choose the smallest positive number whose complete candidate name differs from every current pad name.
Compare full names exactly, before clipping, across all banks in the current project or scratch session.
Include the source, occupied target, missing pads and loading pads in the scan.
Other projects do not reserve numbers. Do not add a saved counter or copy-family identifier.

Scan current names when the owner selects or confirms a target, not only at pickup.
The base name still comes from the pickup snapshot.
A replacement question and the applied copy must use the same proposed name.
If current names change that proposal, ask again before replacing an occupied target.

Cancellation and refusal reserve no number. Undo restores the target and its prior name as one existing undo step.
A cleared or undone copy can free its number for reuse. Numbers are labels, not permanent copy identities.
Saving and reopening keep the stored names. Determine the next available name from the restored pads.

### Compatibility and display

Store the result as the existing custom pad name. Do not change the saved format or sample filename.
Do not rename other pads during copying, loading, saving or reopening.
A source named `kick (copy 1)` keeps its name even if it came from an older project or manual entry.
Only its new copy uses the base-name rule.
Existing name clipping stays unchanged. Long grid labels can still hide the marker.
Use full names for collision scans and storage even when the screen clips them.

## Alternatives

- Keep the source name unchanged: less work, but source and copies have identical names.
- Increase an unmarked final number: this can confuse a copy count with a slice number or a manual name.
- Add `(copy)` repeatedly: the same name-growth problem remains.
- Add persistent copy-family metadata: this changes the saved model without a need in the agreed scope.
- Clean old names automatically: the program cannot distinguish old copy suffixes from intentional numbers.

## Boundary review

The table specifies the required implementation tests. A closed design decision is not a passing test result.

| Situation | Expected result | Status | Planned test |
| --- | --- | --- | --- |
| A copy name exists in another bank | Skip that complete name | Closed in draft | Keyboard and raw MIDI through production routes |
| A source copy has a larger number and a lower number is free | Reuse the smallest free number | Closed by owner decision | Gap and undo scenarios |
| A manual name ends in canonical `(copy N)` | Treat that suffix as the marker for its next copy | Closed by owner decision | Canonical and noncanonical names |
| An occupied target already has a copy number | Include its current name until replacement succeeds | Closed by owner decision | Replacement includes the current target name |
| Names change while a replacement question is open | Ask again if the proposed copy name changes | Closed in draft | Confirmation sequence |
| The source name changes after pickup | Keep the base from the pickup snapshot | Closed in draft | Snapshot sequence |
| The owner cancels or a request fails | Change no pad and reserve no number | Closed in draft | Cancel, refusal and retry sequences |
| The owner saves and reopens work | Keep names and scan restored pads for the next copy | Closed in draft | Production save/load tests |
| An old name contains ` 2 2 2` | Keep those numbers | Closed in draft | Legacy-name preservation |
| A Unicode name is longer than its cell | Keep the full stored name and existing clipping | Closed in draft | Name collision and drawing regressions |

All scenarios need implementation and verification before completion.
Audio callback work, MIDI device measurement and new controls are outside this naming change.

## Implementation

Capture the full displayed name in the existing copy snapshot at pickup.
Use one name calculation for the replacement question and placement.
Remove only a canonical final marker, without parsing its number into a machine integer.
Scan all 128 current pad names on the UI thread. At most 128 names can occupy the numbered candidates.
This bounds the search without a saved counter or new dependencies.

The implementation and its unit tests stay in `crates/app/src/main.rs`.
Recorded keyboard and raw-MIDI scenarios stay in `crates/app/src/input_tests.rs`.
The active delta audit requires real tests for all scenarios.
Keep activation, implementation and scenario tests in one logical commit so that `task check` can pass.
Update user steps in `docs/USER-STORIES.md` and `docs/LIVE-CHECKS.md` in the following documentation task.

## Open questions

The owner resolved all three naming questions with the decisions above.
No naming question remains open.
The owner approved the implementation file plan and activation. No product question blocks implementation.
