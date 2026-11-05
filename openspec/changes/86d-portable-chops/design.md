# Design

## Context

The owner accepted and archived 86b and 86c with their recorded limits.
86d completes the agreed save-safety sequence on `86b-save-retry`. Change 87 is not active.

At `79a9eb5`, `with_stored_paths` and `with_resolved_paths` convert pads, the browser folder and `Session::chops`.
Neither converts `Session::kept` keys. `apply_session` then restores those keys directly into pending older chops.
`focus_epoch` looks up an older chop by resolved file and epoch.
If that lookup fails, existing fallback rebuilds cuts from pad bounds, not from every saved cut.

## Goals / Non-Goals

Preserve every saved variant when the whole project folder moves, including cuts with no assigned pad.
Keep existing path rules, variant retention, pad settings and sound.

Do not copy samples, migrate formats, repair stale paths from an earlier move or add a dependency.
Do not preserve an obsolete unlinked variant beyond the existing retention rules.
Do not change input modes, keys, profiles, callback processing or benchmark exit rules.

## Decisions

### Convert keys at the existing path boundary

Apply the same key conversion to `chops` and `kept` in each path helper.
Use a small local key conversion closure to avoid different path rules between the two maps.
Leave each map value unchanged, including the full vector of older states.
No generic map framework or new storage layer is necessary.

Keep lexical path handling: relative internal paths join the current project folder on load.
External absolute paths remain unchanged. Scratch does not apply project path conversion.
Reject canonicalization here: it changes existing symlink and missing-path semantics.
The same-folder identity rule from 86c is separate from stored sample paths.

### Preserve the existing compatibility boundary

The session version remains 3. An absent `kept` map keeps its current default and pad-bound fallback.
A stale absolute path from an earlier move stays absolute. Do not infer a replacement from a matching file name.
Reject automatic path repair: the saved path does not establish which current sample the owner intended.

### Test through project input and storage

Use isolated project folders and actual WAV files. Seed complete states for several linked variants of one file.
Include at least two older states under the same file key, plus a newest state.
Each older state includes a cut that cannot be recovered from the bounds of its assigned pad alone.
Open, save, move, reopen and save again through the production project and session paths.
Run shared screen entry, save and bound edits through keyboard and raw MIDI with the real profile/dispatcher.
Use keyboard picker navigation in both runs. Do not invent a controller-only picker.
Check both internal and external samples. Check a project without `kept` and one with a stale absolute variant key.

### Compare sound without changing the engine

Render the loaded slice pads before and after relocation through the existing offline audio route.
Compare matching outputs sample for sample. Keep the same pad settings in both runs.
Check finite values, clipping, leading/trailing silence and hit timing within one sample.
Write `renders/86d-portable-chops.wav` with labeled timing in the owner record, not in the audio callback.
This is a storage correction, not a new DSP algorithm or a callback-budget correction.

## Edge cases

| Case | Required result | Status | Check |
| --- | --- | --- | --- |
| Several older variants under one key | Convert the key and retain every state | Closed in automated tests | `portable_chops_keep_all_variants_and_sound`, two older states plus the newest |
| Extra cut with no assigned pad | Preserve the saved cut, not only reconstructed pad bounds | Closed in automated tests | Full-state comparison after F3 in `portable_chops_keep_all_variants_and_sound` |
| Save after relocation, then reopen again | Keep all paths local to the new folder and all states intact | Closed in automated tests | Second production save/open in `portable_chops_keep_all_variants_and_sound` |
| External sample | Keep absolute paths and sample bytes unchanged | Closed in automated tests | Path, byte and no-copy checks in `portable_chops_keep_all_variants_and_sound` |
| Bound edit in an older variant | Change only its linked pads | Closed in automated tests | Keyboard/raw-MIDI edits in `portable_chops_keep_all_variants_and_sound` |
| Old project without `kept` | Keep the existing pad-bound fallback | Closed in automated tests | `portable_chops_keep_old_project_compatibility` |
| Already stale absolute key | Do not guess a replacement path | Closed in automated tests | Pending-state and fallback checks in `portable_chops_keep_old_project_compatibility` |
| Relative paths in scratch | Do not add project resolution | Preserve existing behavior | Existing helper boundary and scratch checks |
| Late sample replies during project open | Keep generation and sound isolation checks | Preserve existing behavior | Existing project-loader tests |

The axes are path location, variant age/count, assigned cuts, move/save order, old data and input route.
No concurrent writers or new crash-durability guarantee enter this change.
Rust tests name all eight delta scenarios. The full check passed 690 App tests and 102 TUI tests.
The 442,368-frame render compares matching before/after slices and checks finite values, clipping, silence and onset within one sample.
The regression failed at the older-variant stored key before the helper correction and passed after it.
Hardware, live keyboard use and listening remain unverified until the owner reports them.
Earlier callback overruns remain unresolved. Callback maximum-budget acceptance stays open.
Keep actual benchmark results and machine observations in ignored local files, not in repository records.
Evidence: `target/86d-code-check.log`, `target/86d-portable-before-fix.log` and `target/86d-portable-after-fix.log`.

## Open questions

None within the agreed scope. Owner acceptance of 86c does not establish manual results for 86d.
