# Design: Pad copy names

## Status and evidence

This is a deferred specification, not an active implementation change.
The owner approved preparation on a separate `87a-copy-names` branch, with implementation after change 87.
The preparation base is `origin/master` at `a4789f4`.
Change 87 remains separate. Its implementation files and acceptance record do not belong to this commit.

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

## Proposed defaults

These defaults make the draft testable. They are not additional owner decisions.
Review the Open questions before activation.

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

The table specifies planned tests. No implementation test for the new behavior exists yet.

| Situation | Expected result | Status | Planned test |
| --- | --- | --- | --- |
| A copy name exists in another bank | Skip that complete name | Closed in draft | Keyboard and raw MIDI through production routes |
| A source copy has a larger number and a lower number is free | Reuse the smallest free number | Requires owner review | Gap and undo scenarios |
| A manual name ends in canonical `(copy N)` | Treat that suffix as the marker for its next copy | Requires owner review | Canonical and noncanonical names |
| Names change while a replacement question is open | Ask again if the proposed copy name changes | Closed in draft | Confirmation sequence |
| The source name changes after pickup | Keep the base from the pickup snapshot | Closed in draft | Snapshot sequence |
| The owner cancels or a request fails | Change no pad and reserve no number | Closed in draft | Cancel, refusal and retry sequences |
| The owner saves and reopens work | Keep names and scan restored pads for the next copy | Closed in draft | Production save/load tests |
| An old name contains ` 2 2 2` | Keep those numbers | Closed in draft | Legacy-name preservation |
| A Unicode name is longer than its cell | Keep the full stored name and existing clipping | Closed in draft | Name collision and drawing regressions |

All draft scenarios remain open for implementation and verification.
Audio callback work, MIDI device measurement and new controls are outside this naming change.

## Open questions

1. May the program reuse a lower number after clearing or undo?
   Default: yes. Choose the smallest free positive number, without a persistent counter.
2. How should copying treat a manual name that ends in canonical ` (copy N)`?
   Default: treat that final suffix as the copy marker. Preserve the manual source name itself.
   This avoids new saved metadata, but the next copy does not retain that suffix as literal base text.
3. Does replacement exclude the target's old name when it selects a number?
   Default: no. Include every current pad name until placement succeeds.
   This avoids an exception in the name scan, but replacement can select a higher number.

These questions do not block storage of this deferred draft. Resolve them before implementation.
