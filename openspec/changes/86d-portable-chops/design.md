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
| Several older variants under one key | Convert the key and retain every state | Open, support now | At least two older states plus the newest |
| Extra cut with no assigned pad | Preserve the saved cut, not only reconstructed pad bounds | Open, support now | Compare complete saved states after F3 |
| Save after relocation, then reopen again | Keep all paths local to the new folder and all states intact | Open, support now | Production save/load round trip |
| External sample | Keep absolute paths and sample bytes unchanged | Open, preserve existing behavior | Path and byte comparisons |
| Bound edit in an older variant | Change only its linked pads | Open, preserve existing behavior | Keyboard/raw-MIDI edit and state comparison |
| Old project without `kept` | Keep the existing pad-bound fallback | Open, preserve existing behavior | Production project open and F3 |
| Already stale absolute key | Do not guess a replacement path | Open, preserve existing behavior | Inspect loaded pending state before fallback |
| Relative paths in scratch | Do not add project resolution | Preserve existing behavior | Existing helper boundary and scratch checks |
| Late sample replies during project open | Keep generation and sound isolation checks | Preserve existing behavior | Existing project-loader tests |

The axes are path location, variant age/count, assigned cuts, move/save order, old data and input route.
No concurrent writers or new crash-durability guarantee enter this change.
Hardware, live keyboard use and listening remain unverified until the owner reports them.
Callback maximum-budget acceptance remains open and has a separate investigation.

## Open questions

None within the agreed scope. Owner acceptance of 86c does not establish manual results for 86d.
