# Tasks: one looped pattern

Implementation started after the owner's apply request. Local signed-off commits are authorized only after green `task check`.
No task is complete until its code, tests, required checks and commit are complete.
Publication, archive, merge and live device access still need separate authorization.

Every completed step needs its own signed-off commit and green `task check`.
Active scenario coverage remains mandatory. Do not add placeholder tests, name-only coverage or temporary exceptions to make a draft pass.
Code and real scenario tests must be present before a commit can pass that gate.
Keep partial implementation uncommitted while the required checks fail.

## 1. Raw model and compilation

- [x] 1.1 Add the checked raw pattern model and immutable playback compilation. Test every grid, strength 0/50/100, midpoint rounding, collision winners and all numeric limits.
- [x] 1.2 Test shorten-before-quantize, seam wrapping and growth without raw-hit loss. Include pad 0, pad 127, empty hits and exactly 4096 hits. Document the accepted file syntax beside the model.

## 2. Session format and migration

- [x] 2.1 Save and load format 5 pattern selection, raw hits and Pitch Gate state for projects and scratch. Test exact round trips and malformed rows, invalid ranges, duplicate slots and mismatched selection through production loaders.
- [x] 2.2 Add fixtures for every supported format 1–4. Test preserved tempo, metronome, names, source links, groups, paths, chops and effects. Compare automated before/after manual renders, including ungrouped overlap.
- [x] 2.3 Test startup read protection, newer-version refusal, failed writes, manual retry and deleted folders with pattern data. Check original bytes and pending work. Add backup and older-binary limits to the owner procedure.

## 3. Engine scheduling and note ownership

- [x] 3.1 Add epoch-tagged pattern commands and sample-or-pattern retirement. Extend real-time allocation/deallocation guards for replacement, full retirement, stale disposal and bounded project restoration. Check ownership recovery without a final callback free.
- [x] 3.2 Schedule hits through the existing frame clock and shared pad trigger. Add automated renders at 40/120/172/300 BPM with 1/128/1024-frame and irregular blocks. Check four two-bar loops, empty blocks, seam events and exact mid-block cuts.
- [x] 3.3 Add note-owned gate and Pitch Gate deadlines with fixed storage. Render positive and zero gate lengths, loop-crossing lengths, capacity recovery, note stealing, same-pad retriggers and both manual/sequenced release orderings.
- [x] 3.4 Implement Stop, restart, tempo change, replacement and pattern removal. Render old deadlines, boundary insertions and unrelated manual, preview, one-shot and effect tails. Compare clock and metronome positions against transport-only control runs.
- [x] 3.5 Test automatic voice-retirement pressure without callback waits, allocation or deallocation. Check dropped-hit counts, eventual UI warning, zero late-hit burst and zero drops in the normal dense fixture. Use an automated render for sound recovery.

## 4. Slice Pitch Gate

- [x] 4.1 Carry the per-pad switch and onset eligibility into sequenced notes. Render linked slices at negative/zero/positive pitch, whole files, detached pieces, manual hits and previews. Check future-note eligibility after source, pitch and switch changes.
- [x] 4.2 Render next-line cuts on every straight and triplet grid, empty steps, Off fallback, strength changes, loop seams and competing ordinary releases. Check existing 5 ms fade, natural source end, unchanged attack and unchanged mute groups.
- [x] 4.3 Extend pad snapshots, copy, clear, reset, scope changes, save and undo for Pitch Gate. Test through keyboard and raw MIDI production routes. Compare automated renders before and after copy and undo.

## 5. Atomic pad swaps and project isolation

- [x] 5.1 Swap all affected raw hit addresses with pad sounds, including hidden and cross-bank hits. Make the audio transaction atomic under command and retirement pressure. Test cancellation, self-target, loading refusal, one-step undo and saved round trips from both inputs.
- [x] 5.2 Render a playing swap and undo. Observe each callback's accepted sound and event mapping. Check stale releases and pattern replacement phase without claiming that the existing swap fade disappears.
- [x] 5.3 Test project switches with patterns, delayed commands and full queues through production loaders and dispatch. Cover empty targets, target read refusal, source save refusal and canonical same-folder selection. Render source fade and target sound isolation.

## 6. Keyboard and controller control

- [x] 6.1 Add the Pads Alt+G action, effective help, owner overrides and startup shadow warnings. Test English/Russian layouts, extended/legacy input, edit open/closed, repeats, releases and higher-priority overlays. Keep existing pages and knob targets unchanged.
- [x] 6.2 Add F10 and F12 learning for the PADS button. Test raw note and CC buttons through current profiles and dispatch, pending confirmation, wrong types, cancel, clear, layer scope, restart and retained owner assignments. Document the text-only pattern-entry exception and unverified SMC-PAD path.

## 7. Accepted audio state and screen feedback

- [x] 7.1 Publish pattern presence and length with the existing coherent epoch-tagged clock snapshot. Test concurrent publication, stale epochs, pending replacements and stopped fallback without a UI clock.
- [x] 7.2 Add loop position, slot identity, `PG on/off`, value feedback, pattern hit flashes, PLAY and splash behavior. Test real callback state without changing selection, held input, banks or source identity.
- [x] 7.3 Extend the offline catalogue for active/stopped/empty/pending patterns, Pitch Gate eligibility and crowded safety states. Run `task screens` at 80 by 26 and 120 by 40. Retain every existing required-text and layout assertion.

## 8. Integration and owner evidence

- [x] 8.1 Add a separate dense-pattern benchmark at 172 BPM, 32 voices, active existing shaping and both effects, 128 frames / 48 kHz. Keep original workloads and limits. Report p99, maxima, exceedance counts and dropped hits for every run.
- [x] 8.2 Deliver `renders/96-pattern-play.wav` and `renders/96-pitch-gate.wav` from automated tests. Check finite samples, no clipping, required silence, level, onset/release timing within one frame and the existing short-cut envelope.
- [x] 8.3 Run `task check`, `task build`, `task screens` and `task openspec:check`. Check that every active scenario names a real executable test. Review the full diff, links and changed English text. Record commands, literal outcomes and unresolved limits here.
- [x] 8.4 Add final keyboard, controller and listening procedures to `docs/LIVE-CHECKS.md`. Update current-capability text only after implementation. Present exact render sections. Task 8.6 keeps owner acceptance separate from the delivered procedures. After the checks, prepare a separate local trial project from an existing owner project with slices. Keep its original files unchanged. Include a ready A1 rhythm and steps to start ptsr, so the owner need not enter TOML. Keep owner audio outside Git and required generated renders.
- [ ] 8.5 After implementation, ask a separate reviewer to follow the owner workflow. Cover manual A1 entry, playback, Pitch Gate, swap, undo, save and reopen. Report useful controls, difficult steps and reproducible defects. Do not access the owner's sound or MIDI devices. Use the documented isolated terminal setup for an interactive check. Keep listening acceptance separate. The owner explicitly requested this delegated review during implementation.
- [ ] 8.6 Obtain owner keyboard, controller and listening acceptance. Resolve the Chop silence interaction in the specification before archive. Do not treat a prepared trial, software input parity or quantitative WAV checks as those results.

## Render and measurement rules

Use generated samples or owner recordings released under CC0. Required tests must not depend on third-party audio or optional demos.
Use distinct low-level generated transients for hit timing. Use a bounded steady generated tone for cut-envelope comparisons.
Compare manual and sequenced control renders to within 0.000001 per sample where operation order is equivalent.
Keep reference amplitudes below clipping without changing production gain or limiting behavior.

Check the first changed frame of a scheduled cut, not only the last audible sample after its fade.
A Pitch Gate fade starts at the grid boundary and can finish after it.
For a source that ends first, check the unchanged source end and silence afterward.
Automatic click checks must compare the intended cut with the existing short-fade reference, not insert an unapproved attack.

Benchmark maxima above 50% remain visible even when mandatory p99 exit checks pass.
A later favorable maximum cannot close the historical callback investigation.
No task authorizes live audio/MIDI device access, commits, push, publication, archive or merge without the applicable owner permission.

## Planning verification

The draft contains 36 requirement blocks and 62 scenarios across seven capabilities. All earlier scenario names in modified blocks remain.

| Command | Result |
| --- | --- |
| `task openspec:check` | Exit 0. `Totals: 17 passed, 0 failed (17 items)` |
| `python3 -B target/96-proposal/verify_docs.py` | Exit 0. Scope, ranges, TOML example, local links, anchors and whitespace pass. |
| `python3 /home/user/.pi/agent/skills/asd-ste100/scripts/ste-lint.py openspec/changes/96-pattern-play/{proposal,design,tasks}.md openspec/changes/96-pattern-play/specs/*/spec.md` | Exit 0. STE-flavored review: 0 hard findings. Passive-voice suggestions remain for states and scenario conditions. |
| `task check` | Exit 201. Format, licenses and Clippy pass. The active-delta audit fails on 51 new scenarios without implementation tests. |

Literal failure:

```text
active scenarios need a named test or an attached scenario description:
```

```text
error: test failed, to rerun pass `-p ptsr --test scenarios`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

Local logs: `target/96-proposal/openspec.log`, `docs-final.log`, `ste-final.log` and `check.log` in the same folder.
This is an incomplete required check, not permission to weaken coverage, commit the draft or call the change complete.
The English review does not claim full ASD dictionary compliance.
Release build, standalone screen generation and the benchmark were not run for this text-only draft.
The failed test gate prevents the benchmark stage of `task check` from running.
No 96 render is ready for listening. Physical-controller checks, listening and historical callback maximum failures remain open.

## Earlier implementation checks

The counts below describe earlier checkpoints, not the current tree. The final implementation section gives the current required-check result.

The raw model and playback compilation for tasks 1.1–1.2 are implemented in `crates/engine/src/pattern.rs`.
The engine exports the new module. The initial model checks below precede the session and scheduler work in the next section.
The seven unit tests check all grids, every strength step, limits, collisions, hidden hits and all 64 loop lengths.
Three active scenarios now have executable coverage. The remaining 48 scenarios still need implementation and tests.
Tasks 1.1–1.2 remain unchecked because the full required gate fails and no commit exists.

| Command | Result |
| --- | --- |
| `task fmt` | Exit 0. `task: [fmt] cargo fmt --all` |
| `cargo test -p engine --lib pattern::tests` | Exit 0. `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.05s` |
| `task check` | Exit 201. Names, formatting, licenses and Clippy pass. Engine tests, input tests and 802 app unit tests pass. The scenario audit fails on the remaining 48 active scenarios. |
| `task build` | Exit 0. ``Finished `release` profile [optimized + debuginfo] target(s) in 16.05s`` |
| `task openspec:check` | Exit 0. `Totals: 17 passed, 0 failed (17 items)` |

The required failure remains:

```text
error: test failed, to rerun pass `-p ptsr --test scenarios`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

Local logs: `target/96-implementation/model/check.log`, `build.log` and `openspec.log` in the same folder.
Focused test and format logs: `target/96-proposal/model-tests.log` and `model-fmt.log`.
The failed audit prevents later workspace tests and the callback benchmark from completing in this run.
Standalone screen checks and new 96 renders remain unverified. No new 96 WAV exists.
No coverage exception, dependency, owner file or factory profile was changed. No commit or publication was made.

### Raw-model boundary review

| Case | Status | Evidence |
| --- | --- | --- |
| Invalid ranges and 4097 hits | Closed | Constructors return errors without clamping. Numeric-limit test passes. |
| Grid midpoint and fractional half-tick | Closed | Integer compilation matches an independent floating-point reference and explicit tie fixtures. |
| Same-pad collisions and different-pad simultaneity | Closed | Deterministic-winner test and unchanged raw rows. Stable sort retains exact ties. |
| Shorten, quantize, grow | Closed | Hidden rows do not wrap. Growing restores the original compiled pattern. |
| Last tick at every supported loop length | Closed | All nine grids and eleven strength values pass at 1–64 bars. |
| Audio scheduling, ownership and queue pressure | Open | Model is not connected to the audio callback yet. |

### Session and scheduler progress

Format 5 now checks full four-integer rows, A1 selection, raw ranges and the saved Pitch Gate boolean.
Project and scratch snapshots retain raw row order and hidden hits. Invalid input cannot replace a saved file.
Migration tests compare manual sound before and after saving formats 1–4. They also check ungrouped overlap.
A malformed five-integer row initially passed fixed-array deserialization. Full-row validation now refuses that input.

The engine now schedules through the existing frame clock. Mixing spans retain the original block interpolation coordinates.
Automatic deadlines identify individual notes. The engine captures gate and Pitch Gate eligibility at onset.
A 64-entry release store expires stale identities. Its overflow test observes the oldest applicable note without releasing a manual voice.
Sample and pattern references use the same bounded retirement queue. Deferred replacements retain the old pattern.
Accepted clock snapshots now include pattern length and dropped-hit count in the same coherent publication.

These changes are partial implementation, not a playable feature in the application.
Application pattern restoration, source eligibility updates, swaps and undo, controls, visible feedback and required renders remain incomplete.
The additional owner-requested workflow review follows implementation. No reviewer ran yet.
All task boxes remain open. No 96 commit exists.

| Command | Result |
| --- | --- |
| `cargo test -p engine --tests` | Exit 0. All engine suites pass, including 19 pattern render tests, 9 callback allocation tests and 14 existing transport tests. |
| `cargo test -p engine --test pattern_play` | The first 19-test run failed one voice-reuse fixture because its repeated manual hits clipped. The full engine rerun now passes. |
| `cargo test -p engine --test pattern_play pattern_deadlines_cannot_follow_a_reused_voice_slot` | Exit 0 after reducing the generated fixture level. The clipping assertion remains unchanged. |
| `cargo test -p engine --test pattern_play pattern_sound_matches_ordinary_pad_playing` | Exit 0. Both channels match within 0.000001 per sample. |
| `cargo test -p engine --lib sequencer::tests` | Exit 0. `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.01s` |
| `cargo test -p engine --test realtime_no_alloc` | Exit 0. `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s` |
| `task lint` | Exit 0. Strict Clippy passed for all workspace targets. |
| `cargo test -p ptsr --bin ptsr` | Exit 0. `test result: ok. 810 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 66.39s` |

The app test run precedes two added pattern save-safety tests.
The later `cargo test -p ptsr --bin ptsr session::` run passes all 15 session tests, including both additions.
The pressure test initially tried recovery with one retirement slot and several triggers per block.
That setup still had pressure. The recovery control now drains the queue between single-frame callbacks and checks no further drops.
A normal dense fixture with production queue capacity remains required.
Logs are local in `target/96-implementation/`, including `pattern-tests-2.log`, `note-reuse.log`, `pattern-rt-tests-3.log`, `lint.log` and `app-tests.log`.
The full required gate, release build, screen catalogue and benchmark still need fresh complete runs.
No new owner WAV or hardware or listening acceptance exists.

| Boundary case | Status | Evidence or remaining check |
| --- | --- | --- |
| Malformed row, invalid data or failed save | Closed in focused tests | Full-row refusal, byte comparison, manual retry, deleted-folder refusal and target-load ordering. |
| Frame boundary, seam and tempo change | Closed in focused tests | Four tempos, four block schedules, empty blocks and pending musical deadlines. |
| Old automatic or manual release after a new note | Closed in focused tests | Both origin orderings, slot reuse and bounded deadline overflow. |
| Ordinary release before the Pitch Gate cut | Closed in focused tests | The short fade preserves its current gain. Coincident and later ordinary releases also pass. |
| Full retirement during replacement or project restore | Closed in focused tests | Allocation and deallocation counts stay unchanged. References survive until the non-audio consumer drains them. |
| Application source edits, swaps, accepted feedback and dense cost | Open | Integration tests and measurements remain required. |

### Application controls and accepted feedback

Pattern restoration, source eligibility updates, Pitch Gate controls and raw-hit-aware swap with inverse undo now have local implementation.
The title shows accepted A1 loop position beside the absolute transport position. Pending lengths and old project snapshots stay hidden.
The pad mode row shows saved Pitch Gate state. Ineligible sources use an inactive style without changing parameter pages or knob targets.
F1 explains eligibility and shows effective keyboard and connected-controller controls.
The catalogue now contains 346 ANSI fixtures, including active, stopped, empty and pending patterns at both supported sizes.
Generated in-memory comparisons cover manual, preview and pattern sound with Pitch Gate on and off.
A keyboard and raw MIDI test checks absolute pad addresses across bank changes and a loop restart.

The first workspace run found a mode-label spacing regression. Normal-width alignment now stays unchanged, and narrow panels retain the complete Pitch Gate label.
The later full run passes all 826 app tests. All existing assertions remain unchanged.

| Command | Earlier result |
| --- | --- |
| `task check` | Exit 201. Names, format, licenses and strict Clippy pass. Engine, input and all 826 app tests pass. Seven active scenarios still lack coverage. |
| `task build` | Exit 0. `Finished \`release\` profile [optimized + debuginfo] target(s) in 41.06s` |
| `task openspec:check` | Exit 0. `Totals: 17 passed, 0 failed (17 items)` |
| `cargo test -p tui` | Exit 0. `test result: ok. 131 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.03s` |
| `task screens` | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 821 filtered out; finished in 72.14s` |
| `cargo test -p engine --test realtime_no_alloc` | Exit 0. Nine tests pass, including retirement pressure, stale commands and recovery. |

The required check still reports:

```text
active scenarios need a named test or an attached scenario description:
error: test failed, to rerun pass `-p ptsr --test scenarios`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

Remaining coverage includes playing swap and undo, copy/clear/file replacement addresses, splash and hit flashes, and three project-transition scenarios.
The benchmark stage did not run because the scenario audit failed. Required 96 WAV files and the dense-pattern benchmark remain absent.
The separate owner-workflow reviewer did not run. Hardware comfort and listening remain unverified.
All tasks remain unchecked and uncommitted until the required gate passes.
Current logs use the `feedback-` prefix in `target/96-implementation/`. The UI style and bank checks also have focused logs there.

| Boundary case | Status | Evidence or remaining check |
| --- | --- | --- |
| Pending length, repeated loop, tempo and old epoch | Closed in focused tests | Production restore and real callbacks supply the title, without UI time estimation. |
| Hidden grid, all parameter pages, both inputs and layouts | Closed in focused tests | Saved state, effective controls, unchanged knobs and 80-by-26 mode text. |
| Pattern title beside ASK, recording and save/audio warnings | Closed in catalogue tests | Existing required labels and the ten-cell PLAY slot remain intact. |
| Positively pitched slice versus file, detached piece, manual hit or preview | Closed in focused tests | Only eligible pattern notes cut. Other On/Off renders match within 0.000001 per channel and sample. |
| Swap during play, project pressure and actual-start feedback | Open | Seven audit scenarios and the remaining integration work above. |

Future recording controls remain plans in `docs/plans/PATTERNS.md` and `docs/plans/SMC-PAD-INTEGRATION.md`.
They do not add recording, button layers or a measured SMC-PAD profile to 96.

### Playing edits and project transitions (earlier checkpoint)

Five more active scenarios now have executable tests through keyboard, raw MIDI CC buttons and raw MIDI notes.
A playing swap and its undo preserve sound and hit mapping as one accepted update under full command or retirement queues.
Both rendered channels match an independent envelope calculation within 0.000001 per sample.
The test observes each accepted swap boundary and the unchanged deadline of an unrelated gate note.
It also checks old deadlines after retrigger and undo, hidden raw hits and zero dropped hits.

Copy, clear, file replacement and their undo preserve raw addresses.
The test checks silence on an empty target and future events after each resulting sound change.
Copy retains the Pitch Gate value from pickup, despite a later edit to its source.
The test found that a completed file load added no undo step, so Undo reverted an earlier edit instead.
A successful current-generation load now records its previous pad snapshot. Project restoration does not add an owner undo step.

Project tests cover invalid data, a file that disappears after listing, a refused source save and a canonical alias of the current folder.
Refusal retains the source clock, sound, raw hits, pending work, undo, picker selection and original file bytes.
Successful switches under both queue pressures fade the source before target readiness.
The target opens stopped. Late source patterns, swaps, Stop commands and releases cannot change the target rhythm.
An empty target remains silent. A target gate note survives beyond the source deadline.

| Command | Current result |
| --- | --- |
| `cargo test -p ptsr --bin ptsr input_tests::patterns -- --nocapture` | The focused 17-test checkpoint passed before the final project-pressure test. The full run below includes all 18 pattern-input tests. |
| `cargo test -p ptsr --bin ptsr pattern_project_switches_reject_stale_work_under_queue_pressure -- --nocapture` | Exit 0. `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 830 filtered out; finished in 2.10s` |
| `task check` | Exit 201. Names, format, licenses and strict Clippy pass. Engine, input and all 831 app tests pass. Two active screen-feedback scenarios still lack coverage. |
| `task build` | Exit 0. `Finished \`release\` profile [optimized + debuginfo] target(s) in 17.46s` |
| `git diff --check` | Exit 0. No output. |
| `task openspec:check` | Exit 0. `Totals: 17 passed, 0 failed (17 items)` |

Literal app result:

```text
test result: ok. 831 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 116.04s
```

Required-check failure at that checkpoint:

```text
active scenarios need a named test or an attached scenario description:
  Sounding pattern notes end the splash without selecting a pad (96-pattern-play/screen-layout)
  Pattern voices light PLAY without pretending to be owner input (96-pattern-play/screen-layout)
```

```text
error: test failed, to rerun pass `-p ptsr --test scenarios`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

Earlier full logs: `target/96-implementation/transactions-check.log` and `transactions-build.log` in the same folder.
Focused logs include `playing-swap.log`, `pattern-edit-tests-3.log` and `pattern-project-pressure-2.log` in that folder.
At that checkpoint, the scenario failure prevented the benchmark stage. The dense-pattern benchmark and required 96 WAV files were absent.
The next section gives the later result. All task boxes remain unchecked until the required evidence and atomic commits are complete.

| Boundary case | Status | Evidence or remaining check |
| --- | --- | --- |
| Swap and undo under either full queue | Closed in focused tests | Real callbacks retain whole updates, matching sounds and hit addresses, and unrelated deadlines. |
| Copy, clear, assignment and undo | Closed in focused tests | Raw rows remain unchanged. Both channels use the resulting sound or required silence. |
| Invalid target or failed source save | Closed in focused tests | Source bytes, pending work, picker, undo and running rhythm remain intact. |
| Same canonical folder | Closed in focused tests | No epoch change, reload, save or clock reset. |
| Delayed source work after target readiness | Closed in focused tests | Target-only sound, stopped restoration and old-deadline isolation under both queue pressures. |
| Actual-start splash, flashes and PLAY | Open | Two active screen-feedback scenarios remain. |
| Dense callback cost, required WAVs and owner workflow | Open | Measurements, delivery and separate acceptance remain required. |

### Actual starts and dense playback (before required WAVs)

The callback now records the first finite, nonzero output of each pattern voice after its gains.
A coherent per-project snapshot retains the accepted start IDs, including notes that finish inside one callback.
The UI uses new IDs for flashes and splash dismissal. It does not change selection, marks, held pads, raw hits or input-source identity.
A separate counter reports omitted hits. The warning appears when the UI observes a new count and does not repeat without another omission.

The new tests cover empty callbacks, missing samples, zero Gate length, empty patterns, zero samples, zero pad or master gain, leading silence and short notes.
Old-project starts cannot cross the ready barrier. Keyboard, raw MIDI CC and raw MIDI note tests cover hidden pads, EDIT and another open bank.
One first-frame test failed with `assertion failed: stats.pads_playing(0)` at -48 semitones.
The accepted output now lights PLAY before the fractional source position reaches one whole frame. The regression test passes.

The separate dense benchmark retains the original steady and switch workloads and their exit checks.
It adds a two-bar loop with exactly 4096 events: 32 pads at every 1/64 line.
The run uses 172 BPM, pitch, envelopes, filters, drive, both sends, reverb, delay and metronome at 128 frames and 48 kHz.
It measures 20,000 callbacks. Retirement drains outside each measured callback.
Its additional exit check rejects wall p99 above 50% or any dropped hit.

| Command | Latest result |
| --- | --- |
| `task check` | Exit 0. Names, format, licenses, strict Clippy, workspace tests, scenario coverage and benchmarks pass their command gates. |
| `task build` | Exit 0. `Finished \`release\` profile [optimized + debuginfo] target(s) in 25.92s` |
| `cargo test -p ptsr --bin ptsr input_tests::patterns -- --nocapture` | Exit 0. `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 813 filtered out; finished in 8.34s` |
| `cargo test -p engine --lib rt::tests` | Exit 0. `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 2.27s` |
| `cargo test -p engine --test realtime_no_alloc` | Exit 0. Nine tests pass. The full run also includes these allocation and deallocation guards. |
| `cargo test -p ptsr --test scenarios` | Exit 0. All three audits pass. The two missing active scenarios now have executable tests. |

Literal app result from the latest full run:

```text
test result: ok. 834 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 188.07s
```

Latest measured percentages of the 2.667 ms callback budget:

| Workload | Wall p99 / maximum / count above 50% | Thread CPU p99 / maximum / count above 50% |
| --- | --- | --- |
| Original steady playback | 21.54 / 54.13 / 1 | 21.55 / 53.22 / 1 |
| All project-switch callbacks | 7.79 / 26.37 / 0 | 7.80 / 26.43 / 0 |
| Dense pattern playback | 14.50 / 33.56 / 0 | 14.51 / 33.59 / 0 |

Dense-pattern dropped hits: zero. The per-switch wall peak p99 is 25.92%.
The original steady workload exceeds the maximum target once despite its passing p99 gate.
This run does not establish the cause or close earlier maximum-target failures. No performance correction is part of this step.
Logs: `target/96-implementation/activity-dense-check.log`, `activity-dense-build.log`, `activity-rt-low-pitch.log` and `activity-no-alloc.log`.
Callback CSV: `target/bench-diagnostics/callback-807596-1794155133968202597-0.csv`.
The earlier standalone dense run also passed, but does not replace these full-run results.

| Boundary case | Status | Evidence or remaining check |
| --- | --- | --- |
| Short, silent or not-yet-processed pattern note | Closed in automated tests | Only actual nonzero output publishes a start. Short completed notes retain their event. |
| Low-pitch first frame | Closed in automated tests | Audible output lights PLAY before a whole source frame advances. |
| Selection, bank, marks, held pads and input identity | Closed in automated tests | Real keyboard and raw MIDI paths retain each owner state while sequenced notes flash. |
| Project change or stale notification | Closed in automated tests | Only current ready-project starts update the UI. |
| Full retirement queue and omitted hits | Closed in automated tests | One warning per new count, no delayed replay, no repeated flash after recovery. |
| Maximum callback target | Open | One original steady callback exceeded 50% in wall and thread CPU time. |
| Required WAVs, owner trial and independent workflow review | Open | No new WAV, prepared trial or separate reviewer result exists yet. Listening and physical controls remain unverified. |

No commit exists. Task completion still requires the remaining evidence and atomic commits.

### Required listening files

`crates/engine/tests/pattern_renders.rs` now creates both required WAV files from generated transients and steady tones.
It compares both channels against independently timed manual commands within 0.000001 per sample.
It checks finite samples, no clipping, level, silence and onset timing within one frame.
Pitch Gate checks retain the attack prefix and compare the first changed cut frame against the existing short-cut reference.
Manual Pitch Gate On and Off renders match. The saved-file inspection also checks the float subtype, channels, sample rate and actual data.

| File | Duration | Listening sections |
| --- | --- | --- |
| `renders/96-pattern-play.wav` | 14.900 seconds | 0–8 seconds: two two-bar loops at 120 BPM. 8–11.2 seconds: third loop at 150 BPM. 11.2–14.4 seconds: fourth loop with later snare hits after pattern replacement. Each loop includes gate bass notes with an 80 ms release. Final 0.5 seconds: silence. |
| `renders/96-pitch-gate.wav` | 7.500 seconds | 0–2.5 seconds: linked +12-semitone slice, Pitch Gate On. 2.5–5 seconds: the same pattern with Pitch Gate Off. 5–7.5 seconds: equivalent manual hits with Pitch Gate On, matching Off. Each section includes the loop seam at its local 2 seconds and a closing short fade at 2.25 seconds. |

The first Pitch Gate cut starts at local 0.125 seconds despite the next recorded note arriving at 1.9375 seconds.
The last note crosses the local two-second loop seam. The next loop starts there and cuts at local 2.125 seconds.
The test checks the unchanged attack, required silent intervals and matching short-cut behavior through the seam.
These checks do not establish listening acceptance. No owner audio or third-party recording enters these files.

Focused command: `cargo test -p engine --test pattern_renders -- --nocapture`.
Literal result:

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s
```

Focused log: `target/96-implementation/required-renders-2.log`.
Saved-file checks: `target/96-implementation/required-wav-files.log`.
Full checks after the new render tests: `target/96-implementation/renders-check.log` and `renders-build.log`.
`task check` exits 0. Its app result is `test result: ok. 834 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 129.66s`.
`task build` exits 0 with `Finished \`release\` profile [optimized + debuginfo] target(s) in 0.11s`.
OpenSpec validation reports `Totals: 17 passed, 0 failed (17 items)`.
`task screens` exits 0: `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 829 filtered out; finished in 76.88s`.
It writes 346 ANSI files at 120 by 40 and 80 by 26. Log: `target/96-implementation/renders-screens.log`.
Saved-file peaks are 0.06929134 for pattern playback and 0.04000000 for Pitch Gate. Both files use stereo float32 at 48 kHz.

This run reports dense-pattern wall p99 8.34%, maximum 18.03%, and zero callbacks above 50%.
Its thread CPU p99 is 8.35%, maximum 18.04%, with zero above 50%. Dropped hits remain zero.
Original steady playback reports wall p99 9.97%, maximum 61.78%, with one callback above 50%.
Its thread CPU p99 is 9.93%, maximum 29.72%, with zero above 50%.
The per-switch wall peak p99 is 10.13%. The earlier CPU and wall maximum failures remain open.
These observations do not establish a cause. CSV: `target/bench-diagnostics/callback-821709-1794156151135577900-0.csv`.
A separate owner trial, final manual procedures and the independent workflow review remain incomplete.

## Final local trial checkpoint

The preceding implementation sections retain historical checkpoints. This section describes the delivered local tree.
Core playback, persistence, controls, state feedback and executable scenario coverage have local signed-off commits.
The dense benchmark and required generated renders have separate commits. Owner procedures complete task 8.4.
Task 8.5 remains open only for its blocked interactive terminal pass. Its independent code and offline workflow review is complete.
Task 8.6 and the unresolved maximum-budget investigation remain open. Nothing is published, archived or merged.

| Command | Literal result |
| --- | --- |
| `git fetch --all --prune` with non-interactive SSH and a 45-second outer limit | Exit 0, no output. The earlier timeout is superseded. |
| `task check` | Exit 0. `test result: ok. 835 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 140.11s` for the app. The workspace passes 1226 tests. |
| `task build` | Exit 0. ``Finished `release` profile [optimized + debuginfo] target(s) in 0.09s`` |
| `task screens` | Exit 0. `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 830 filtered out; finished in 92.91s` |
| `task openspec:check` | Exit 0. `Totals: 17 passed, 0 failed (17 items)` |
| `python3 -B target/96-implementation/final/verify_delivery.py` | Exit 0. `Local Markdown targets and anchors: 13 PASS` and both WAV frame counts and SHA256 values pass. |
| `python3 -B target/96-implementation/check_trial_launcher.py` | Exit 0. Valid project and scratch paths pass. External roots, samples, chop keys and scratch references fail closed. |
| `./target/96-owner-trial/start.sh --check` | Exit 0. `Trial paths checked. Existing owner projects and settings are not selected.` |
| STE lint on the changed English text | Exit 0, zero hard findings. States retain relevant passive wording. Procedures use Strict review, descriptions use STE-flavored review. No full dictionary claim. |

Logs and exit files are under `target/96-implementation/final/`.
The screen catalogue contains 346 ANSI files. The active change contains 36 requirements and 63 scenarios.
The scenario audit remains intact, with no new hardware exception or placeholder.
The source tree passed the required gate before the local commits. Later artifact edits do not change runtime code.

| Workload | Wall p99 / maximum / count above 50% | Thread CPU p99 / maximum / count above 50% |
| --- | --- | --- |
| Original steady playback | 14.91% / 39.43% / 0 | 14.92% / 39.47% / 0 |
| All project-switch callbacks | 8.46% / 11.14% / 0 | 8.46% / 11.16% / 0 |
| Dense pattern | 7.40% / 14.99% / 0 | 7.34% / 15.00% / 0 |

Dense playback drops zero hits. Per-switch wall peak p99 is 10.94%.
Local CSV: `target/bench-diagnostics/callback-880197-1794161916479989711-0.csv`.
This favorable run does not close the earlier wall maximum of 61.78% or the earlier CPU maximum failures.

### Independent workflow findings

The read-only reviewer inspected production code, real keyboard/raw-MIDI tests, persistence, required renders and the complete owner procedure.
It found useful one-chord Pitch Gate, effective help, stable edit pages, one-step swap undo and retained save/reopen state.
Awkward paths remain explicit: A1 entry requires a closed program, previews have their own Space role and flashes belong to the visible bank.
No recording or step editor was added to hide those intermediate limits.

The reviewer found that older format numbers could carry new pattern data. A failing loader test reproduced acceptance of version 1 with A1.
The loader now refuses pattern selection or entries under formats 1–4 without discarding them.
All ten session tests pass, including target refusal before source saving and startup write protection.
The reported tick-zero loss was false. The reviewer withdrew it after checking the unclamped search boundary.
`replacement_and_swap_before_the_first_frame_keep_tick_zero` passes without a scheduler change.

Chop entry fades current voices, but future pattern events continue while transport runs.
`chop_entry_hush_does_not_stop_future_pattern_events` reproduces that behavior through keyboard, raw CC and raw note routes.
It also checks that Space remains preview and that returning to Pads permits Stop.
This is a characterization of the local trial, not resolution of the earlier solo-silence wording.
The open question and current default are in `design.md`. Task 8.6 blocks archive until resolution.

The interactive terminal pass was not run. A pre-existing owner ptsr graph node prevented the required empty-graph condition.
Namespace isolation failed with permission errors. Kernel Landlock preflight passed, but did not waive that condition or establish a device result.
The reviewer had no shell tool and ran no tests. Parent results above and focused logs provide the executed test evidence.
No second sampler, audio client or MIDI reader was started.

### Prepared trial and owner project cleanup

`target/96-owner-trial/start.sh` opens a copied, stopped project with eight owner slice pads and a ready two-bar A1 at 92 BPM.
Its binary, settings, projects, scratch and logs are local. It copies no saved owner MIDI bindings or external downloads.
Master starts at -12 dB, metronome off. Pad W is intentionally +12 semitones and named `PG +12`.
The source project and copied recording hashes matched at preparation and verification. Required WAVs contain only generated sound.
The owner must save and close an existing ptsr instance before starting the trial.
The agent checked the launcher paths and shell syntax, not its live audio or MIDI behavior.
The owner then reported a successful trial after the instructions to start the trial and press Space.
This is owner evidence for basic startup and prepared-pattern playback only.
Pitch Gate, swap, undo, saved state, physical controls and generated WAVs still need owner checks.
Pattern recording remains a later change, not an implemented feature of 96. Task 8.6 stays open.

The owner's later cleanup request changed six closed older project headers to format 5 with their existing transport defaults made explicit.
Original bodies match their backups byte for byte. The already-current open project was not written.
A read-only helper compiled the current production loader and accepted all seven project files without opening a device.
No new conversion path, audio move, automatic A1 insertion, slice edit or setting change was added.
Local evidence: `project-cleanup.log`, `project-cleanup-backup.txt` and `project-loader-check.log` under `target/96-implementation/`.

Repository rules and future recording-control plans are retained in separate repository-only branches, not mixed with the 96 implementation.

### Build cache recovery

The owner later reported that the normal build failed. `task build` reproduced 18 compiler errors, including:

```text
error[E0432]: unresolved import `engine::Retired`
error[E0432]: unresolved import `engine::pattern`
```

Cargo resolved all dependencies from the main checkout. No other Cargo or rustc process was active.
The verbose build marked `engine`, `input` and `tui` as `Fresh`, but the engine dependency file omitted both new pattern modules.
The cause was a shared target directory used for earlier checks in repository-only worktrees.
Their older library artifacts remained after the successful 96 checks. The delivery build claim did not cover this later cache state.

Cleaning the four workspace packages without `--release` changed only debug artifacts and left the release failure unchanged.
Cleaning the same packages with `--release` made the normal build compile all four packages successfully.
No runtime source, owner project, dependency or device setting changed during recovery.
The resulting release binary has the same SHA256 as the prepared trial binary.
Do not share this target directory with other worktrees. Check the ordinary main-checkout build after all worktree operations.

| Command | Literal result |
| --- | --- |
| `task build` after release-cache cleanup | Exit 0. ``Finished `release` profile [optimized + debuginfo] target(s) in 18.86s`` |
| `task check` | Exit 0. `test result: ok. 835 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 136.43s` for the app. |
| `./target/96-owner-trial/start.sh --check` | Exit 0. `Trial paths checked. Existing owner projects and settings are not selected.` |
| `python3 -B target/96-implementation/check_trial_launcher.py` | Exit 0. All isolated-path and refusal controls pass. |

Logs are in `target/96-build-recovery/`: `before.log`, `clean.log`, `build.log`, `clean-release.log`, `build-release.log`, `check.log`, `trial-check.log` and `trial-launcher-tests.log`.
Dense playback reports wall p99 8.79%, maximum 25.89%, thread CPU p99 8.80%, maximum 12.74%, and zero dropped hits.
This passing run does not close historical maximum-target failures or establish listening and controller acceptance.

## Workflow follow-up

- After owner acceptance and authorization, sync all seven deltas into the main specifications and archive in the same PR.
- Check that archive synchronization retains unrelated requirements and that every resulting main scenario has a real test.
- Run all required checks on the archived result before authorized publication or merge.
- Keep the branch after an authorized exact-head squash merge. Preserve the PR title and final signoff.
