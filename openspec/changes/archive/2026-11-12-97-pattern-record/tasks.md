# Tasks

Keep all implementation boxes open until required checks pass and the corresponding commit exists.
Recording passes the functional checks and ordinary `task check`. The additional callback maximum check still fails.
Implementation boxes still require their separately authorized commits.
The earlier verification sections below retain historical failures. Use the completion summary for the latest software result.
Use [the design](design.md) and the eight delta files as the contract.
Each completed logical step needs its substantive tests, relevant documentation, a green `task check` and a signed-off atomic commit.
Do not bypass the active-scenario audit or add name-only tests to make a planning commit green.

## 1. Capture model and pass boundaries

- [ ] 1.1 Add bounded capture events, pass identities and coherent accepted phase feedback. Test stale epochs, empty blocks, repeated commands and full queues without callback allocation or deallocation.
- [ ] 1.2 Add complete Gate capture and bounded terminal recovery. Test seam duration, tempo changes, retrigger, pass-end closure, lost release feedback and the 4096-row limit through offline renders.
- [ ] 1.3 Extend sequencer eligibility for recorded rows. Select eligible collision rows before the existing winner order. Preserve old eligible hits and independent manual sound. Render forward quantize, wrapped zero, delayed publication, grid changes, pending Gates and Stop/restart against independent manual references. Check both channels without doubled or replayed occurrences. Generate `renders/97-pattern-repeat.wav`.

## 2. Count in and lifecycle

- [ ] 2.1 Add one-bar audio-timed count-in, forced temporary clicks and exact tick-zero start. Render cancellation, warm-up holds, tempo changes and metronome preference preservation.
- [ ] 2.2 Handle record end, Stop and interrupted audio through recoverable boundaries. Test completion before clock reset, a missing final callback and later restart without stale capture.
- [ ] 2.3 Add App-side capture draining and pattern publication. Test accepted empty pads, unavailable audio, capture overflow and preservation of the original raw row order.
- [ ] 2.4 Replace same-step hits at pass completion. Use the pass's final quantize and strength. Test the newest row at one pad and tick and two fresh rows of one pass. Test other ticks, other pads, hidden rows, quantize Off, project load and pass Undo. Use keyboard and raw MIDI. Check `renders/97-pattern-record.wav` again and report whether its sound changes.

## 3. Keyboard and production MIDI paths

- [ ] 3.1 Add the new actions, default keys and shadow warnings without changing owner entries. Test English/Russian and extended/legacy events, repeats, releases, `none` and old microphone overrides.
- [ ] 3.2 Add recording and erasure flags to shared production routing. Test keyboard, raw note and learned CC routes across screens, banks, overlays and deferred releases. Preserve existing source-specific text and learn behavior.
- [ ] 3.3 Keep incompatible placement, marking and assignment out of active recording. Test refusal in both directions with unchanged pads and a working recovery notice.
- [ ] 3.4 Add separate pattern and audio-capture guards. Test microphone/output conflict orders, refused microphone release and absence of WAV or port-link requests from pattern actions.
- [ ] 3.5 Add real-pad routing and drawing on Chop during pattern work. Test unassigned slices, linked slices, preview exclusion, release ownership and entry without a recording-phase hush.

## 4. Pattern settings and learned controls

- [ ] 4.1 Add compact Bars, Quantize and Strength controls on every screen. Test clamping, hidden rows, phase refusal, raw timing and return to the underlying edit page. Add current help with this step.
- [ ] 4.2 Add GLOBAL command buttons, parameter buttons and named absolute knobs to F12 MIDI. Test confirmation, cancellation, clear, restart, range mapping and per-device jump/pickup/scale through production dispatch.
- [ ] 4.3 Test complete command-before-hit workflows with keyboard, raw MIDI notes and learned CC buttons. Count command presses and separate learned software paths from unmeasured physical SMC-PAD behavior.

## 5. Erasure and one-pass Undo

- [ ] 5.1 Add bounded audio-time erase targets and retained removals. Render silent targeted onsets, seams, collision rows, delayed UI and released targets. Check that hidden rows and other pads remain.
- [ ] 5.2 Add safe erase entry, exit and no-release fallback. Test older ordinary holds, bank changes, one-loop expiry, Stop and return to sounding pads without a synthetic hit.
- [ ] 5.3 Add isolated pattern pass history. Test overdub Undo, erasure and grid restoration, empty passes and consecutive passes without time merging. Preserve unrelated pad, FX and Chop edits.
- [ ] 5.4 Add global pattern Undo and active-pass completion. Test every screen, normal Pads Ctrl+Z, newer-swap refusal and ordinary local Chop Undo. Document the lack of redo.

## 6. Save and project safety

- [ ] 6.1 Defer autosave during active phases and complete capture before manual save or quit. Test pending Gate completion, write refusal, read protection, missing folders and retry through keyboard and raw MIDI.
- [ ] 6.2 Guard project picker entry during active phases. Test unchanged source clock and take on refusal, then normal same-folder and target-validation behavior after completion.
- [ ] 6.3 Round-trip completed takes through production project and scratch serialization. Check format 5, exact raw rows, saved settings and stopped Idle restoration without runtime fields. Do not edit owner projects.

## 7. Visible state and owner material

- [ ] 7.1 Add accepted COUNT, REC A1, ERASE, Finishing and compact settings feedback. Clarify existing Pitch Gate feedback as Off, On for A1 only, or On but inapplicable with a source or pitch reason. Test selection and source/pitch changes without changing the saved switch or claiming cuts from manual hits or transport. Check current-project truth, timer progress, hidden pads, edit states, long names and safety priorities at 80×26 and 120×40.
- [ ] 7.2 Extend the existing catalogue matrix and effective-key help. Cover Pitch Gate states, text restrictions and the visible manual-play exclusion through keyboard and raw MIDI, including mixed A1 playback. Add substantive assertions for the new scenarios, not only the earlier state-and-color scenario. Run `task screens` and check every applicable screen/mode pair, blocked state, overridden key and no-release hint without devices.
- [ ] 7.3 Generate `renders/97-pattern-record.wav` from permitted sounds. Include raw take, quantized repeat, overdub, erasure and Undo. Check both channels, finite samples, clipping, level, silence, onset timing and cuts against independent references.
- [ ] 7.4 Update README and `docs/LIVE-CHECKS.md` with exact keyboard, learned-controller and listening procedures. Check links and English text. Mark unavailable hardware, listening and live terminal checks as unverified.

## 8. Integration and delivery

- [ ] 8.1 Review each delta scenario against a substantive production-path test. Run `task check`, retain original callback workloads and add a separate dense recording/erasure workload without changing targets or exit gates.
- [ ] 8.2 Run `task openspec:check` and `task screens`. Run ordinary main-folder `task build` after all other worktree operations. Record literal results and remaining limits. Keep Cargo targets separate per worktree.
- [ ] 8.3 Review the complete diff and the boundary table again. Check that owner-project edits, factory-profile claims, new dependencies, migrations and unrelated DSP changes are absent.
- [ ] 8.4 Deliver a usable owner trial only after recording works and checks pass. Give separate keyboard/controller steps and the checked WAV. Record owner results without converting general acceptance into individual physical or listening claims.

## Local completion verification

The owner approved the ten-file release fix and nine additional completion paths.
All active scenarios now have substantive tests. The audit itself stays unchanged.

- `task check` passes. App reports `993 passed; 0 failed`. Input reports `87 passed; 0 failed`. Notices report `25 passed; 0 failed`.
- The scenario audit reports `3 passed; 0 failed`. The 28 callback allocator tests and 20 benchmark-report tests pass.
- TUI reports `133 passed; 0 failed`. Separate `task screens` reports `10 passed; 0 failed`.
- Strict OpenSpec reports `Totals: 18 passed, 0 failed (18 items)`. Its informational archive warning remains separate. No archive was attempted.
- Ordinary main-folder `task build` passes. Its literal result is ``Finished `release` profile [optimized + debuginfo] target(s) in 17.93s``. No shared worktree target was used.
- The final `renders/97-pattern-record.wav` contains 1344000 stereo float32 frames at 48000 Hz. All 19 attacks match the independent reference within one frame.

The final workflow passes through keyboard, raw notes and learned CC. It covers first recording, quantized repeat, overdub, erasure, grid changes and one-pass Undo.
Additional checks cover count-in cancellation, rejected input, audio-block rounding, Gate retrigger and clamping, overflow and the raw-row limit.
Every grid and strength 0, 50 and 100 has an independent future-playback comparison, with unchanged raw rows.
Rendered checks retain effective edit bindings, explicit `none`, arrow-only controls, long project names and persistent safety status.
README and the [owner procedure](../../../docs/LIVE-CHECKS.md#97-record-and-erase-a1) describe the new disposable trial and listening file.

Active recording and erasure now have separate 20000-callback workloads. Their checks retain 32 sounding pads, accepted counts, complete Gates and zero losses.
Each workload reports release and the accepted Finishing boundary separately. The new maximum check cannot hide a rare wall or CPU overrun.
Original playback, dense-pattern and switching workloads and exit checks remain unchanged.
The historical 61.07 percent wall overrun has a matching paired trace. It separates CPU time from the elapsed-time gap but does not identify a responsible process.
Earlier CPU-budget overruns remain unresolved. Later passing runs do not close that investigation.
The additional `taskset -c 12 task bench` diagnostic fails the new active erasure gate: CPU maximum 62.64 percent, with ten blocks above 50 percent.
The CPU 0 comparison also retains original playback overruns, despite exit zero under its original p99 gate.
Profiler preflight fails with restricted performance-monitoring access. No privilege or kernel setting was changed.
The callback maximum requirement remains unsatisfied. No runtime timing fix is selected.

Exact before/after files, hashes, complete bounded diffs, commands and literal failures remain in `target/97-completion/`.
These snapshots overlap prior work and are not independent commit patches. All 29 boxes remain open until their commits exist.
No commit, publication, archive, merge, owner-project edit or live device access occurred.
Physical controllers, terminal reporting, kitty graphics and listening remain unverified.

## Approved F1 readability work

The owner approved wrapped F1 rows, Alt+PageUp/PageDown and learned GLOBAL page commands.
This work belongs to tasks 3.1, 4.2 and 7.2. It does not complete those tasks by itself.
Follow the boundary and file set in [the design](design.md#approved-f1-readability-boundary).
Implement shared wrap and viewport bounds, independent tab offsets, effective controls and an accepted recording caption.
Test visible pages through keyboard and raw MIDI at both sizes on all current screens.
Include owner overrides, repeats, releases, blockers, Messages, resize and changed content.

The implementation now wraps the key table and shows the effective page keys, position and accepted recording caption.
Six focused App tests pass. They inspect every wrapped row on all current screens at both sizes.
They also test 128 erasure targets, both keyboard protocols, raw notes, learned CC, tab offsets and first-frame command order.
Separate column checks retain long key text, descriptions, wide characters and combining characters.
The existing F12 learning check now includes both page commands through confirmation, cancellation, clearing and restart.
Offline audio checks keep an original Gate sounding through blocked page commands and stop it on its original release.
The first complete run found two unavailable page commands in ordinary Alt help.
The implementation now keeps them only in F1. The original Alt-row assertion passes unchanged.

The final required run passes all 975 App tests, 87 input tests and 25 notice tests.
The active scenario audit still fails with 23 uncovered recording scenarios. The benchmark stage does not run after this failure.
Separate checks pass: eight screen tests, 133 TUI tests, release build and strict OpenSpec validation.
Local commands, literal results, before-files and the complete boundary diff remain in `target/97-f1-pages/`.
The final recording WAV and owner trial remain open. No implementation box is complete.

Keep the scenario audit unchanged. Close a scenario only after a substantive production-path test passes.
Physical Alt+Page reporting, learned hardware controls and listening remain unverified.
No commit, publication, live device access, new dependency or saved-format change belongs to this boundary.

## Raw MIDI erasure workflow checks

Continue tasks 4.3 and 5.1 with the [complete raw input workflow](design.md#complete-raw-midi-erasure-workflow-checks).
Check command-before-target order, a direct release at tick 240 and three loops without UI collection.
Use notes and learned CC on all current screens. Compare both audio channels against an independent manual reference.
Retain the release-boundary row, hidden rows and other pads. Check delayed collection, one-step pass Undo and unchanged routing.
Keep these checks open until they pass. Do not change the scenario audit or mark implementation tasks complete.

Initial workflow result, before the approved fix:

Both new tests initially failed on production MIDI release routing. A command and its following pad press in one batch retained UI-only release ownership.
Without another UI update, the target stays active and also removes the release-boundary row. Notes and learned CC reproduce this defect.
A control lets App consume the command before the press. It clears the target correctly.
Note Off encoding and queued erasure-off do not change the failure.
The temporary diagnostic no longer exists. The two permanent tests retain the regression and the complete workflow.
The full required run reports `975 passed; 2 failed` for App. Names, format, licenses and Clippy pass.
The scenario audit and benchmark stages do not run after these failures. The last completed audit still lists 23 uncovered scenarios.
Release build passes in `0.10s`. Strict OpenSpec reports `18 passed, 0 failed`. `git diff --check` passes.
The complete reference comparison, three-loop delay and pass Undo remain blocked by the release assertion.
See `target/97-erase-workflows/` for commands, control results and the preserved three-file boundary.

The owner approved the [ten-file release fix](design.md#approved-release-fix).
The design specified the runtime-only commands before code. Existing commands and ordinary manual release behavior remain unchanged.
The implementation adds no saved-format, factory-profile, dependency or input-gain change.

### Approved release fix results

Eight focused raw-input tests pass through production MIDI and the offline callback.
The original workflow now completes its independent stereo reference, three-loop UI delay and one-step Undo checks on all five screens.
Additional tests cover early release, disconnect, repeated controls, ports, channels, deferred bank changes and grid publication.
Old project envelopes and old-pass releases cannot change a new target. A late erase release cannot cut a fresh ordinary Gate.

All 28 callback allocator tests pass. They include full queues, all 4096 handles, stale input, project reset and ordinary Gate retirement.
The first complete integration run passes all 980 App tests, 87 input tests and 25 notice tests.
Its active audit still lists 20 scenarios without an attached substantive test. The benchmark stage does not run after that failure.
Later focused tests cover old projects, old passes and a pending release before one callback.
The first pending-release regression failed because the old hold consumed a fresh Gate's release obligation.
Only accepted ordinary handles now consume that UI obligation. The eight focused checks pass after this fix.
The second integration run retains the earlier pending-release failure. A later format check refused the unformatted regression.
The final required run passes all 983 App tests, 87 input tests and 25 notice tests.
Its active audit lists 19 uncovered scenarios and stops `task check` with exit 201. The benchmark stage does not run.
Main-folder release build passes in 0.07 seconds. Strict OpenSpec reports 18 passed, with no failures.
The separate screen run passes all eight tests. Production code stayed unchanged after that screen build.
Commands, literal logs, before-files and hashes remain in `target/97-erase-release/`.

The unchanged benchmark passes separately. Steady wall maximum is 30.35 percent, dense playback is 25.15 percent, and switching is 34.27 percent.
No measured callback exceeds 50 percent in those runs. This does not explain the historical 61.07 percent maximum or measure active recording.

Keep all implementation boxes open. The final WAV, remaining scenarios, owner instructions and callback investigation remain separate unfinished work.
Hardware, listening and physical terminal behavior remain unverified. This fix authorizes no commit or publication.

## Planning verification before implementation

This section records planning checks only. It does not complete an implementation task.
The active scenario audit must remain unchanged and red until substantive tests cover the new behavior.
Do not commit a step while the mandatory `task check` fails.

- `task openspec:check` passed: `Totals: 18 passed, 0 failed (18 items)`.
- `python3 target/97-proposal/verify_docs.py` passed. It checks 11 Markdown files, eight capability paths, two local links and retained scenario names.
- English prose lint reports `0 hard`. The prose review kept 21 passive-voice suggestions because each actor is clear or irrelevant.
- `task build` passed for unchanged implementation in `0.08s`. Its literal output is below and in `target/97-proposal/build.log`.
- `task check` failed with Task exit 201 after Cargo test exit 101. Names, format, licenses and Clippy passed.
- Existing App unit tests report `835 passed; 0 failed`. The active-delta audit reports 65 new recording scenarios without tests.

The build reports:

```text
    Finished `release` profile [optimized + debuginfo] target(s) in 0.08s
```

The failed audit reports:

```text
active scenarios need a named test or an attached scenario description:
```

Its final result is:

```text
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.98s
error: test failed, to rerun pass `-p ptsr --test scenarios`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

Full logs and the new-file patch stay local under `target/97-proposal/`.
No implementation, test or main specification changed. No commit, publication or live device access occurred.
The benchmark stage did not run after the test failure. New recording screens and the 97 WAV do not exist yet.

## Capture protocol work in progress

The owner approved the seven-file plan for task 1.1 on 2026-11-09.
The engine now provides a 512-event capture queue and explicit pass start, end and acknowledgement commands.
Events retain accepted ticks, absolute pad, velocity, mode, epoch, pass and sequence.
The end snapshot remains available when the queue fills.
Acknowledgement requires the matching final sequence and a drained queue.

The new tests cover stale projects, readiness, repeats, empty blocks, rounding, queue pressure and Stop before reset.
A control comparison checks both output channels for every sample under capture pressure.
Allocator counters check both allocation and deallocation through the real callback.
The publication test checks concurrent reads across different epochs and pass boundaries.

The implementation does not yet complete Gate durations, run count-in or publish captured rows into A1.
The App does not enable this queue yet. Keyboard, learned MIDI, erasure, Undo and saving remain later tasks.
No new WAV exists for owner listening. Task 7.3 retains that deliverable.
The intermediate protocol tests do not claim complete coverage of any unfinished owner scenario.

Verification for the partial capture protocol:

- `cargo test -p engine --test pattern_capture --test realtime_no_alloc` passed.
  Each suite reports `11 passed; 0 failed`.
- `task check` passed names, format, licenses and Clippy, then failed the active-scenario audit.
  The App suite reports `835 passed; 0 failed`. The audit still lists 65 recording scenarios without named tests.
  Existing test names, audit rules and scenarios remain intact.
- `task openspec:check` reports `Totals: 18 passed, 0 failed (18 items)`.
- `cargo test -p tui` passed separately because the failed audit stopped the workspace run before this suite.
  It reports `131 passed; 0 failed`.
- `task bench` returned zero with unchanged workloads and exit checks.
  Steady wall p99 is 11.98 percent, but one block reached 61.99 percent against the 50 percent target.
  Steady thread CPU maximum is 27.33 percent. Dense playback reports zero dropped hits.
  This run does not close the maximum-budget failure or establish its cause.
  It does not measure active recording. Task 8.1 retains the separate recording and erasure workload.
- `task build` passed from the main repository folder in 18.32 seconds.
- English prose uses STE-flavored review. Structural lint results are in `target/97-capture/ste.log`.
  This review does not claim full ASD dictionary compliance.

The release build reports:

```text
    Finished `release` profile [optimized + debuginfo] target(s) in 18.32s
```

The failed required check reports:

```text
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.25s
error: test failed, to rerun pass `-p ptsr --test scenarios`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

Logs stay local under `target/97-capture/`.
Task 1.1 remains open because the required `task check` fails and no commit exists.
No commit, publication, owner-project edit or live device access occurred.

## Gate capture work in progress

The owner approved task 1.2 with the ten-file plan.
The engine now retains bounded Gate completion data separately from the 512-event feedback queue.
Recovery requires the queued onset and matching project, pass, sequence and slot.
Release, retrigger and pass end close known Gates. Stop uses the accepted boundary before clock reset.
The compiled source-row count includes hidden and collision rows.
Accepted onsets reserve space immediately. The next onset after 4096 rows remains audible but unrecorded.
Queue omissions and row-limit refusals have separate counters.

This step adds 12 tests: eight Gate integration tests, two identity tests, one source-count test and one allocator test.
The allocator test accepts all 4096 Gate slots, fills feedback, refuses another onset and completes the pass.
The existing project-reset allocator test now starts with a pending Gate.
Sound comparisons preserve both channels through release loss, retrigger, row refusal, end and Stop.

The first Gate test run reported seven passes and one level-check failure.
That fixture measured only the first 128 frames of the existing 20 ms envelope glide.
The fixture now measures 960 frames before retrigger. The level threshold and reference comparison stay unchanged.
The focused rerun passes.

Verification for the engine boundary:

- `cargo test -p engine --lib --test pattern_capture --test pattern_capture_gates --test realtime_no_alloc` passed.
  The suites report 24, 11, eight and 12 passes, with zero failures.
- `task check` passed names, format, licenses and Clippy. All executed ordinary tests passed.
  The App suite reports `835 passed; 0 failed`.
  The active audit still fails: 63 scenarios need tests, down from 65 after two substantive Gate scenarios gained tests.
  The audit, existing scenarios and earlier test names remain unchanged.
- `cargo test -p tui` passed separately: `131 passed; 0 failed`.
- `task bench` returned zero with unchanged workloads and exit checks.
  Steady wall p99 is 12.85 percent and maximum is 39.55 percent.
  Dense playback maximum is 41.24 percent and dropped hits are zero.
  All measured maxima in this run stay below 50 percent. This does not resolve historical maximum failures.
  Active recording and erasure performance remain unmeasured until task 8.1.
- `task build` passed from the main repository folder in 34.54 seconds.
- `task openspec:check` passed: `Totals: 18 passed, 0 failed (18 items)`.
- `cargo test --workspace --doc` passed. All three documentation suites contain zero tests.
- English prose uses STE-flavored review. Structural lint results stay in `target/97-gates/ste.log`.
  This review does not claim full ASD dictionary compliance.

The failed required check reports:

```text
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.69s
error: test failed, to rerun pass `-p ptsr --test scenarios`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

The release build reports:

```text
    Finished `release` profile [optimized + debuginfo] target(s) in 34.54s
```

The local file `renders/97-pattern-capture-gates.wav` lasts 6.9375 seconds at 48 kHz with two channels.
Both channels contain finite samples, peak 0.031496063 and maximum step 0.000227078.
Measured onsets are frames 90001, 207001 and 303001, within one frame of their expected positions.
The final 4800 frames are silent. Production-engine playback matches an independent manual reference.
Listen for a shorter first hold, then two equal longer holds after the pause.
Check each release for clicks. Owner listening remains unverified.
This file does not demonstrate keyboard recording, overdub, erasure or Undo. Task 7.3 retains that complete demonstration.

Logs stay local under `target/97-gates/`.
The App does not enable or drain capture yet. Verification did not use live devices or owner projects.
Erasure, space reuse, saving and audio-loss recovery remain later tasks.
Task 1.2 stays open because `task check` fails and no authorized commit exists.
No commit or publication occurred.

## Approved playback rules for task 1.3

The owner approved the first-repeat and eligible-collision rules after the sampler-manual review.
Manual hits stay immediate. Fresh rows do not add same-lap sequenced triggers.
A hit quantized across the seam first repeats at the next eligible loop start.
An ineligible fresh collision cannot suppress an eligible old hit.
Once rows are eligible, change 96's velocity, gate-length and raw-order rules select one sequenced hit.
Manual sound remains independent of the old pattern, including nearby hits on the same pad.

The patterns delta now includes two collision scenarios. Both still need substantive production-engine tests.
The design records the accepted behavior, rejected replacement behavior, source manuals and boundary cases.
This edit changes only the three agreed OpenSpec documents. It adds no implementation or test coverage.
Task 1.3 and all other implementation boxes remain open.

Verification of this specification edit:

- `task openspec:check` reports `Totals: 18 passed, 0 failed (18 items)`.
- The English prose review uses STE-flavored mode. Structural lint reports `3 violations (0 hard, baseline 0)`.
  The three retained passive-voice suggestions concern unchanged scenarios with clear actors. This is not full ASD dictionary compliance.
- Text checks preserve all earlier scenario names. They check links and open implementation boxes.
  Rust sources and build configuration match their pre-edit checksums.
- `task check` passes names, format, licenses, Clippy and the executed ordinary tests, including `835 passed; 0 failed` for App.
  The active-scenario audit fails with 65 uncovered scenarios, including the two new collision scenarios.
  Its result is `test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.27s`.
  Task exits with 201 and reports `task: Failed to run task "check": task: Failed to run task "test": exit status 101`.
  The benchmark stage does not run after this failure.

Logs and the document diff stay local under `target/97-eligibility-spec/`.
No release build or listening check ran for this text-only edit. No commit, publication or live device access occurred.

## Recorded playback work in progress

The owner approved the eight-file plan and one WAV for task 1.3.
The engine now retains a transport run and the accepted capture lap before raw tick rounding.
Recorded compilation uses checked runtime identities and retains all collision candidates.
Eligibility precedes winner selection. Replacements retain consumed occurrences and exclude elapsed occurrences.
Pending Gates stay silent. Stop and restart clear the earlier run's temporary restrictions.
The saved four-integer rows and ordinary compilation stay compatible.

This step adds 17 tests: 16 playback integration tests and one allocator test.
The playback tests use production project queues and compare both channels with independent manual sound.
They cover forward quantize, seam neighbours, late publication, grid changes, collisions, pending Gates and restart.
They also cover project reset, empty callbacks, slot generations and the first transition from ordinary playback.
The allocator test exercises 4096 collision candidates, a full retirement queue, replacement and restart.
App publication must initialize existing identities under the current grid before its first in-pass grid edit.
The design records that requirement for task 2.3.

The first new suite reported 11 passes and two failures.
At frame 96000, the clock reported tick `3839.999999997088`. The new lap calculation selected the previous lap.
Capture origins now use the sequencer's nearest-audio-frame seam instead of an integer-tick round or a strict floating-point floor.
Tests cover the seam and both one-frame neighbours at 40, 120, 172 and 300 BPM.
The other failure counted tone zero crossings as new attacks.
Pulse checks now detect even adjacent pulses. Tone checks use the generated sample length to separate attacks.
Both-channel sample comparisons retain the original 0.000001 tolerance.
Diagnostic logs also retain a corrected compile error and the WAV-format inspection.

Verification for the engine boundary:

- `cargo test -p engine --lib --test pattern_capture --test pattern_capture_gates --test pattern_record_playback --test pattern_play --test realtime_no_alloc` passed.
  In command-target order, the suites report 24, 11, eight, 16, 21 and 13 passes, with zero failures.
- `task check` passed names, format, licenses, Clippy and all executed ordinary tests.
  App reports `835 passed; 0 failed`. The active audit fails with 60 uncovered scenarios, down from 65.
  Five recording scenarios gained substantive tests. Audit rules, scenario names and previous tests remain intact.
- `cargo test -p tui` passed separately: `131 passed; 0 failed`.
- `cargo test --workspace --doc` passed. The three documentation suites contain zero tests.
- `task bench` returned zero with unchanged workloads and exit checks.
  Steady wall p99 is 15.09 percent and maximum is 23.59 percent.
  Dense playback maximum is 19.98 percent with zero dropped hits.
  This run stays below the 50 percent maximum target. It does not resolve historical budget failures.
  Active recording, runtime collision candidates and erasure performance remain unmeasured until task 8.1.
- `task build` passed from the main repository folder in 17.97 seconds.
- `task openspec:check` reports `Totals: 18 passed, 0 failed (18 items)`.
- English prose uses STE-flavored review. Final lint and text-check results stay in `target/97-repeat/`.
  This review does not claim full ASD dictionary compliance.

The required check reports Task exit 201 and:

```text
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.38s
error: test failed, to rerun pass `-p ptsr --test scenarios`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

The release build reports:

```text
    Finished `release` profile [optimized + debuginfo] target(s) in 17.97s
```

The local `renders/97-pattern-repeat.wav` lasts 4.375 seconds at 48 kHz with two float32 channels.
It contains a generated 110 Hz left tone and a 165 Hz right tone. No external sample is necessary.
The onsets are frames 5040, 102000 and 198000, at 0.105, 2.125 and 4.125 seconds.
Channel peaks are 0.035888772 and 0.037025608. Maximum steps are 0.000542343 and 0.000895994.
Both channels contain only finite samples. The final 4800 frames are silent.
The production result matches the independent manual reference within 0.000001 per sample.
Listen for three short tones. The first starts immediately, without another attack 20 ms later.
The next two tones follow the loop grid, two seconds apart. Check the attacks and tails for clicks.
Owner listening remains unverified. This is not the complete task-7.3 recording demonstration.

Logs and the step diff stay local under `target/97-repeat/`.
The App still does not enable or drain capture. Count-in, input controls and saving remain later tasks.
Task 1.3 stays open because the required check fails and no authorized commit exists.
No commit, publication, owner-project edit, live audio run or device-setting change occurred.

## Approved Pitch Gate feedback clarification

The owner heard no difference while toggling Pitch Gate during manual play.
The existing sound contract deliberately leaves manual hits unchanged.
The owner approved a clearer indicator and text explanations in change 97, without changing that contract.

The proposal, design and screen-layout/screen-help deltas now require:

- Distinguish Off, On for A1 only, and On but inapplicable.
- Explain a source or pitch restriction in text, not color alone.
- Keep the manual-play exclusion visible, including mixed manual and A1 playback.
- Do not infer a sounding cut from the saved switch, running transport or a manual hit.
- Retain the current mode row, frame, safety priorities, edit pages and effective controls.

The design proposes compact labels `PG off`, `PG A1` and `PG n/a` with explicit help texts.
Exact compact wording remains an open question with these defaults. The three-state distinction has owner approval.
Tasks 7.1 and 7.2 include this future implementation and its named tests.
The earlier scenario name remains, but its old state-and-color test does not establish the new feedback.
Four additional scenarios require substantive UI and input tests. They remain unimplemented.

Verification of the specification edit:

- `task openspec:check` reports `Totals: 18 passed, 0 failed (18 items)`.
- `python3 target/97-pitch-gate-spec/verify_docs.py` passes.
  It checks five documents, 10 requirement descriptions, four links and the unchanged set of 29 open implementation boxes.
  Earlier scenario names remain. The delta retains the earlier Pitch Gate feedback scenario and adds four new names.
  All 72 Rust source and configuration files match their pre-edit checksums.
- English prose uses STE-flavored review. Structural lint reports `2 violations (0 hard, baseline 0)`.
  Both retained passive-voice suggestions concern unchanged text or an existing scenario name.
  This review does not claim full ASD dictionary compliance.
- `cargo test -p ptsr --test scenarios` exits with 101.
  It reports `FAILED. 2 passed; 1 failed` and 64 uncovered active scenarios, including the four new Pitch Gate scenarios.
  The audit and test sources remain unchanged. This step adds no name-only coverage or weaker audit rule.
- `git diff --check` exits with zero.

Logs and the document diff stay local under `target/97-pitch-gate-spec/`.
The full `task check`, code build, screen generation and listening check did not run for this text-only edit.
The earlier full check remains failing. Do not treat the new scenario declarations as implementation or acceptance.

This step edits only five OpenSpec documents. It changes no code, sound, bindings, project format or dependency.
No new listening WAV is necessary. The existing `renders/96-pitch-gate.wav` remains the comparison reference.
Implementation boxes remain open. No commit, publication or live device access occurs in this specification step.

## Pitch Gate feedback work in progress

The owner approved the nine-file implementation plan.
The mode row now shows `PG off`, `PG A1` or `PG n/a`.
The third existing Pads help row explains the selected pad's setting and unchanged manual sound.
The first two rows retain compact commands. F1 retains the complete effective-key help.
Learning, questions, safety indicators, edit pages and popup priorities keep their existing rules.
The drawing snapshot now uses one four-state value instead of separate switch and eligibility fields.
Source restrictions take precedence over pitch restrictions. Setting changes do not toggle the saved switch.
No audio command, saved format, binding, dependency or engine file changed in this step.

Four new named scenario tests cover restrictions, manual/A1 scope, fixed frames and inaudible manual toggles.
The existing parameter-page test now checks the new feedback through keyboard, raw notes and CC buttons.
A fifth new test checks the complete popup text and target at both terminal sizes.
The initial compile failure omitted the existing `linked_slice` field in a test command pattern.
The first frame test compared hidden pads with a shown-pad reference. It now compares the same frame configuration.
Neither correction weakens the expected state, row position or sound result.

The new popup test first failed at 80x26 because the value field clipped the long restriction.
Popup values now fit the existing 26-cell limit. The full-text assertion remains and passes.
Manual-toggle tests cover whole files, linked slices at zero pitch and eligible linked slices with edit open and closed.
Their On/Off audio blocks match exactly in both channels. Separate tests retain manual tails past the pattern grid deadline.

Verification and logs stay local under `target/97-pitch-gate-ui/`.

- `cargo test -p ptsr --bin ptsr pitch_gate -- --nocapture` reports `14 passed; 0 failed` after the final test expansion.
- `task screens` reports `6 passed; 0 failed`, with 354 base ANSI entries and 192 additional Pitch Gate frames.
- `cargo test -p tui` reports `132 passed; 0 failed`.
- `cargo test --workspace --doc` passes. Its three documentation suites contain zero tests.
- The final `task check` passes names, format, licenses, Clippy and all executed ordinary tests.
  App reports `839 passed; 0 failed`. The active audit fails with 60 uncovered scenarios, down from 64.
  All four new Pitch Gate scenarios have substantive named tests. The audit and scenario names remain unchanged.
  The final audit reports `test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.93s`.
  Task exits with 201 and reports `task: Failed to run task "check": task: Failed to run task "test": exit status 101`.
- `task bench` returns zero with unchanged workloads and exit checks.
  Steady wall p99 is 14.88 percent and maximum is 31.50 percent. Dense playback maximum is 20.79 percent with zero dropped hits.
  These results do not resolve historical maximum-budget failures or measure active recording and erasure.
- Main-folder `task build` passes. Its literal result appears below.
- `task openspec:check` reports `Totals: 18 passed, 0 failed (18 items)`.
- `python3 target/97-pitch-gate-ui/verify_docs.py` checks the nine approved files, text, links and 29 unchanged open boxes.
  All 68 source/configuration files outside the approved list match their pre-step checksums.
- English descriptions use STE-flavored review. Procedures use Strict mode.
  The changed-prose lint reports `0 violations (0 hard, baseline 0)`. This is not full ASD dictionary compliance.

The release build reports:

```text
    Finished `release` profile [optimized + debuginfo] target(s) in 20.93s
```

The screen catalogue adds eight base state/size entries and 192 state, edit, visibility and safety frames.
[The owner procedure](../../../docs/LIVE-CHECKS.md#97-pitch-gate-setting-and-applicability) separates keyboard, controller and listening checks.
No new WAV is necessary. `renders/96-pitch-gate.wav` remains the sound comparison.
The 96 owner-trial script uses a copied binary. Its old labels do not establish the new build's behavior.

Tasks 7.1 and 7.2 remain partial: recording indicators, controls and their full catalogue are later work.
All implementation boxes remain open while the required check fails and no authorized commit exists.
The App still does not enable or drain capture. This feedback boundary is not usable pattern recording.
Owner listening, physical-controller use, a live terminal pass and kitty image composition remain unverified.
No commit, publication, live device access or owner-project edit occurred.

## Count-in work in progress

The owner approved the nine-file plan for task 2.1 and `renders/97-pattern-count-in.wav`.
The patterns delta defines Stop, Play, transport toggle and repeated requests during CountIn.
The engine now uses a separate audio clock for four temporary metronome beats.
A stopped start requires an accepted A1, capture setup and a fresh pass identity.
At the nearest-frame end boundary, playback and capture start once at zero with a new run identity.
Count-in ticks and the remaining beat number use the existing coherent recording snapshot.
The saved metronome preference stays unchanged. Explicit preference changes control only playback after count-in.
Cancellation retains warm-up voices and their ordinary releases. Play cancels preparation and starts playback without recording.
Project reset clears the timer and rejects old or unready recording commands.

The step adds 13 tests: 11 integration/render tests and two allocator tests.
The sound tests use production project queues and independent metronome/manual-pad references for both channels.
The timing matrix crosses 44.1, 48 and 96 kHz with 40, 120, 172 and 300 BPM.
It uses 1, 128, 1024 and mixed callback lengths, plus starts inside a callback.
Other tests cover changed tempo, preference changes, cancellation next to zero, warm-up Gates and all remaining beat values.
New transport runs, repeated requests, project epochs, 32 manual voices and full retirement queues also have tests.
The existing prerequisite test now checks accepted stopped starts and rejection of a cancelled pass identity.

The first test failed because a stopped start still returned Idle. It passed after engine implementation.
The first complete new suite reported eight passes and one fixture failure.
The fixture compared tick `719.9999999999251` with `720.0` using exact floating-point equality.
It now checks accepted fractional time within 0.00000001 ticks and separately requires saved integer tick 720.
Both-channel sound comparisons retain the original 0.000001 tolerance.
The independent WAV reader initially expected the short IEEE-float header.
Inspection identified WAVEFORMATEXTENSIBLE with the IEEE-float subtype. The reader now checks that subtype and 32-bit samples.

Verification and literal logs stay local under `target/97-count-in/`:

- `cargo test -p engine --lib --test pattern_count_in --test pattern_capture --test pattern_capture_gates --test pattern_record_playback --test pattern_play --test transport --test realtime_no_alloc -- --nocapture` passes.
  In execution order, the eight suites report 24, 11, eight, 11, 21, 16, 15 and 14 passes, with zero failures.
  The later full check also passes the expanded 11-test count-in suite in 29.80 seconds.
- `task check` passes names, format, licenses, Clippy and every executed ordinary test.
  App reports `839 passed; 0 failed` in 143.31 seconds.
  The active audit fails with 58 uncovered scenarios, down from 60 despite two additional transport scenarios.
  Four count-in/transport scenarios now have substantive coverage. The audit and earlier scenario names remain intact.
  The full cancellation scenario still needs App Undo and input coverage. The engine tests do not claim those unfinished parts.
- `task bench` exits with zero under the unchanged p99 exit checks, but does not meet every maximum target.
  Steady playback reports wall p99 15.08 percent and maximum 35.52 percent.
  Dense playback reports wall p99 9.27 percent, maximum 73.22 percent and one block above the 50 percent maximum target.
  Dense thread CPU maximum is 34.70 percent. Dropped hits are zero.
  The elapsed-time excess has no established cause. Do not claim that this step caused or resolved it.
  Active count-in, recording and erasure timing remain unmeasured. Task 8.1 retains that separate workload.
- `cargo test -p tui` reports `132 passed; 0 failed`.
- `cargo test --workspace --doc` passes. Its three suites contain zero tests.
- `task openspec:check` reports `Totals: 18 passed, 0 failed (18 items)`.
  It also notes that the change-97 Pitch Gate delta cannot archive before change 96 supplies its requirement header.
  This step does not archive either change.
- Main-folder `task build` passes. Its literal result appears below.
- `python3 target/97-count-in/verify.py` checks the nine approved files, 680 unchanged files, text and 29 unchanged open boxes.
  It retains earlier scenario names and checks local links, fences, newlines, whitespace and the generated WAV independently.
- English descriptions use STE-flavored review. Listening procedures use Strict mode.
  Structural lint results stay in `target/97-count-in/ste.log`. This is not full ASD dictionary compliance.

The required check reports Task exit 201 and:

```text
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.25s
error: test failed, to rerun pass `-p ptsr --test scenarios`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

The release build reports:

```text
    Finished `release` profile [optimized + debuginfo] target(s) in 17.12s
```

The local WAV has 480000 stereo float32 frames at 48 kHz and lasts 10 seconds.
Channel peaks are 0.5. Maximum adjacent steps are 0.096080772.
All samples are finite. Each section ends with at least 4800 silent frames.
Independent attack checks stay within one frame. Both channels match the independent engine reference within 0.000001 per sample.
Use [the listening procedure](design.md#fourth-implementation-boundary-audio-timed-count-in) for the three sections.
Owner listening, live terminal and physical-controller checks remain unverified.

The partial step changes three engine files, three test files and three OpenSpec documents. It adds one local WAV.
Keyboard, MIDI, App capture publication, Undo, saving and recording feedback remain later tasks.
All 29 implementation boxes remain open because the required check fails and no authorized commit exists.
This boundary is not usable pad recording. No commit, publication, device access or owner-project edit occurred.

## App capture boundary in progress

The owner approved the original seven-file App publication boundary and one local listening WAV.
The owner then approved three additional engine files for publication that preserves sounding notes.
The contract is in [the sixth implementation boundary](design.md#sixth-implementation-boundary-app-capture-publication).
Task 2.3 remains open. The approved App and engine implementation is present, with focused tests and a local WAV.
Recording keys, learned recording controls, Undo, saving and live devices remain outside this boundary.

The initial test command was:

```text
cargo test -p ptsr --bin ptsr app_capture_records_accepted_empty_pads_once_in_audio_order -- --nocapture
```

Before implementation, it reported `FAILED. 0 passed; 1 failed` because the accepted phase was Idle instead of Recording.
That result shows the missing App capture setup, not the sounding-note blocker.
The complete log is `target/97-app-capture/red.log`, with Cargo exit 101.
The earlier unconnected draft remains in `target/97-app-capture/draft/`.
At that stop point, the source matched the pre-step backups. The new module now connects capture collection in the App.
Other earlier dirty work remains unchanged.

A separate controlled render shows that ordinary pattern replacement ends an existing Gate early:

```text
left: frame 1024, control = 0.02, same-pattern replacement = 0
right: frame 1024, control = 0.02, same-pattern replacement = 0
PASS: ordinary replacement ends an existing Gate before its 960-tick release; control keeps it sounding.
```

The render experiment exits 0. Its source and log stay under `target/97-app-capture/`.
The design records the approved three-file engine extension and rejected ordinary-replacement alternative.
The owner authorized implementation without enabling recording controls.
The new `PublishRecordedPattern` command preserves accepted Gate and Pitch Gate deadlines.
Ordinary pattern replacement retains its previous behavior. Stale or unready publication retires its pattern outside the callback.
The App collects accepted events, retains pending Gate identities and queues publication before terminal acknowledgement.
Audio interruption retains the last accepted boundary without publication or acknowledgement to the stopped instance.
Saved rows and project format 5 remain unchanged.

Evidence and literal command output remain in `target/97-app-publication/`:

- `cargo test -p ptsr --bin ptsr app_capture_ -- --nocapture`: `14 passed; 0 failed`.
  The render test crosses three production input routes, timely/delayed collection and completed-pass swaps.
  Other tests cover old Gate tails, pending rows, overflow, row limits, Stop, project restoration and audio loss.
- The focused engine command covers the library and seven capture, playback and allocator suites.
  Its suites report 27, 11, 8, 10, 11, 21, 16 and 18 passed, with zero failures.
  Allocator tests include 4096 descriptors, 32 sounding voices, full retirement and obsolete publication.
- `task check`: the App suite reports `853 passed; 0 failed`.
  The active-scenario audit still lists 58 uncovered scenarios. The required check exits 201.
  Recording controls and complete save/Undo/exit behavior remain absent. Do not weaken the audit for this partial boundary.
- `task lint`: Clippy passes with `-D warnings`. `task fmt` passes.
- `task bench`: exit zero under its unchanged p99 checks, but one dense-pattern block exceeds the 50 percent maximum.
  Steady wall p99/max are 14.51/38.60 percent. Dense wall p99/max are 9.43/56.92 percent.
  Dense thread CPU maximum is 26.45 percent. Dropped hits are zero.
  The cause remains unknown. Active recording, publication and erasure workloads remain unmeasured.
- `cargo test -p tui`: `132 passed; 0 failed`. The three workspace documentation suites pass with zero tests.
- `task openspec:check`: `18 passed, 0 failed`. The existing change-96 archive prerequisite remains informational.
- `task build`: the release build passes in the main folder.
- `python3 target/97-app-publication/verify.py`: ten approved files changed, 681 other files unchanged and all 29 boxes open.
  It checks local links, anchors, fences, final newlines and whitespace.
- `python3 target/97-app-publication/inspect_wav.py`: both channels match an independent analytic reference within 0.000001 per sample.
  It checks finite samples, clipping, levels, attacks within one frame, the reference click bound and required silence.

The local WAV is `renders/97-pattern-app-capture.wav`: 336000 stereo float32 frames at 48 kHz, seven seconds.
Use [the listening procedure](design.md#app-boundary-verification-and-listening) for the manual hit, later repeats and overdub over the previous rhythm.
Owner listening and live-device checks remain unverified.
This boundary does not connect session dirty tracking, save, exit or Undo. It does not enable recording controls.
The final `task check` rerun reports `853 passed; 0 failed` in 141.72 seconds for the App.
The active audit still lists 58 uncovered scenarios and reports:

```text
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.77s
error: test failed, to rerun pass `-p ptsr --test scenarios`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

Descriptions use STE-flavored review. The listening procedure uses Strict mode.
The structural lint reports zero findings. This is not full ASD dictionary compliance.
The required check is not green. All implementation boxes remain open, and no commit or publication occurred.

## Interrupted capture work in progress

The owner approved the five-file engine recovery plan for task 2.2 and `renders/97-pattern-interrupted.wav`.
The implementation adds `Recovery::interrupted(onset, boundary)` in `crates/engine/src/capture.rs`.
It returns raw hits from the last coherent accepted snapshot without a fabricated audio acknowledgement.
A stable pending Gate closes at that boundary. A stale or changing slot returns no row.
Known Gate durations cannot grow or cross the supplied boundary. One-shot duration stays zero.
The existing live `Recovery::completed` method still returns no pending Gate.
No new callback write, queue, saved field, dependency or binding exists.

New coverage includes ten integration/render tests, two atomic unit tests and one allocator test.
Production queues model a queued finish request with no final callback, then destroy the isolated engine.
Retained Stats and recovery storage remain readable without a fake acknowledgement.
Other cases cover released and retriggered notes, loop seam, tempo change, Stop/restart, new pass/project and invalid boundaries.
A full 512-event queue omits a release and onset. Recovery retains only the accepted onset and its known duration.
Atomic tests distinguish stable pending data from odd revisions, stale identities and concurrent writes.
Allocator counters cover all 4096 accepted slots, repeated recovery, end, Stop and restart without allocation or deallocation.

The initial new test could not compile before the method existed. Cargo exited 101 and reported:

```text
error[E0599]: no method named `interrupted` found for struct `Arc<Recovery>` in the current scope
error: could not compile `engine` (test "pattern_capture_recovery") due to 24 previous errors
```

The first integration run passed all ten tests. The expanded focused run also passed.
No test or tolerance needed a correction for this boundary.
Evidence and literal command output remain in `target/97-interrupted/`:

- `cargo test -p engine --lib --test pattern_capture_recovery --test realtime_no_alloc -- --nocapture`:
  three suites report 26, 10 and 16 passed, with zero failures.
- `task check`: names, format, licenses, Clippy and the executed ordinary tests pass.
  The App suite reports `839 passed; 0 failed` in 144.86 seconds.
  The active audit still lists 58 uncovered scenarios. Full interrupted saving and Undo remain uncovered because App integration is absent.
- `task bench`: exit zero under the unchanged p99 checks, but not all maxima meet the 50 percent target.
  Steady wall p99/max are 15.32/24.87 percent. Dense wall p99/max are 8.64/52.35 percent, with one block above target.
  Dense thread CPU maximum is 16.82 percent. Dropped hits are zero.
  The cause remains unknown. Active recording and erasure workloads remain unmeasured.
- `cargo test -p tui`: `132 passed; 0 failed` in 7.06 seconds.
- `cargo test --workspace --doc`: three zero-test suites pass.
- `task openspec:check`: `18 passed, 0 failed`.
  The existing informational archive warning still needs the change-96 `Visible Pitch Gate state` header first.
  This step does not archive either change.
- `task build`: the release binary builds in the main folder.
- `python3 target/97-interrupted/verify.py`: five approved files changed, 685 other files unchanged and all 29 task boxes retained.
  It checks local links/anchors, fences, newlines, whitespace and the generated WAV independently.
- English descriptions use STE-flavored review. The listening procedure uses Strict mode.
  The final lint reports zero violations, including zero hard findings.
  Structural lint output stays in `target/97-interrupted/ste-final.log`. This is not full ASD dictionary compliance.
  The first lint flagged mixed `check`/`confirm` wording and misread `has ten` as a compound tense.
  Revised wording removes those findings without a change to facts.
- `git diff --check`: no whitespace errors.

The required check reports Task exit 201 and:

```text
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.98s
error: test failed, to rerun pass `-p ptsr --test scenarios`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

The release build reports:

```text
    Finished `release` profile [optimized + debuginfo] target(s) in 18.85s
```

The local WAV has 216000 stereo float32 frames at 48 kHz and lasts 4.5 seconds.
The known short Gate lasts 125 ms. The recovered Gate repeats at 1 and 3 seconds with equal 250 ms duration.
Both channels match independent manual commands within 0.000001 per sample.
Independent inspection checks finite samples, peaks below 1, adjacent steps below 0.004, attacks within one frame and release/silence windows.
Use [the listening procedure](design.md#fifth-implementation-boundary-interrupted-capture-recovery) for the exact sections.
Owner listening remains unverified.

This boundary is not complete task 2.2 or usable pad recording.
App audio-loss detection, row publication, Undo, saving/exit and recording controls remain later work in tasks 2.3, 5.3 and 6.1.
The new tests do not establish the complete interrupted-save scenario.
All 29 implementation boxes remain open because the required check fails and no authorized commit exists.
No commit, publication, live device access or owner-project edit occurred.

## Completed-pass Undo work in progress

The owner approved the six-file plan and `renders/97-pattern-pass-undo.wav`.
The contract is in [the seventh implementation boundary](design.md#seventh-implementation-boundary-completed-pass-undo).
Implement isolated common-history steps for completed passes and ordinary Undo of that state.
Preserve unrelated pads, FX and Chop work, accepted note releases and the existing history bound.
Test fresh row identities after Undo, empty or cancelled passes, repeated completion, queue pressure and project/audio lifetime.
The required check remains red. No implementation box is complete or authorized for commit.
Recording controls, active-pass/global Undo, erasure and complete save/exit behavior remain outside this boundary.

Implementation and focused verification:

- A requested start retains the prepared A1 snapshot until the matching audio acceptance.
- A changed terminal pass adds one isolated common-history step after row and Gate completion.
- Ordinary Undo compiles the snapshot before changing the model or removing history.
- Live restoration preserves current note deadlines. Interrupted restoration sends no command to the stopped instance.
- Each new row uses its accepted pass number as its runtime generation. Pending Gate completion retains that generation.
- Completion collection precedes a later common edit and compilation of a later pad swap.
- The 13 new tests cover history order, no-op passes, 4096-row refusal, note tails, interruption, project reset and queue pressure.
- The earlier 14 App capture tests also pass.

The required `task check` stopped before the scenario audit:

```text
test result: FAILED. 865 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 145.90s
error: test failed, to rerun pass `-p ptsr --bin ptsr`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

Its exit code is 201. The failure reads a shared temporary keymap file:

```text
called `Result::unwrap()` on an `Err` value: Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

Six keymap tests use the same process-specific temporary directory.
The isolated group reproduces five failures, including foreign keymap contents and the missing file.
The originally failed test passes alone. Both helpers and all six callers match the pre-Undo snapshot.
This separate test-isolation problem remains unchanged. Do not skip it or describe the required check as green.

The separate scenario audit reports 57 uncovered active scenarios and exits 101.
Its result is `FAILED. 2 passed; 1 failed`. The completed-overdub scenario now has its substantive production-input test.
Erasure, dedicated cross-screen Undo and complete persistence scenarios remain unimplemented.

Other checks:

- The eight focused engine suites pass: 27, 11, 8, 10, 11, 21, 16 and 18 tests.
- TUI tests pass: 132 tests. Workspace documentation tests pass with no tests in their three suites.
- Names, format, licenses and Clippy pass inside `task check` before its test failure.
- `task bench` exits 0. Steady wall p99/max is 18.92/46.01 percent. Dense wall p99/max is 16.01/33.94 percent.
- Dense playback has 4096 hits, 32 voices and no dropped hits. Both benchmark phases have no wall block above 50 percent.
- This run does not explain earlier maxima above 50 percent. Active recording and Undo workloads remain unmeasured.
- OpenSpec validation passes: 18 items. The existing change-96 archive prerequisite remains informational.
- Main-folder `task build` passes in 20.16 seconds.

The local render is `renders/97-pattern-pass-undo.wav`: 432000 frames, 9 seconds, stereo float32 at 48000 Hz.
Both channels pass manual-reference and separate analytic checks within 0.000001 per sample.
Finite samples, clipping, reference level, one-frame attacks, click bound and final silence also pass.
The listening procedure is in [the design](design.md#completed-pass-verification-and-listening).
Owner listening and physical-controller acceptance remain unverified.

Detailed commands, literal results and retained failures are in `target/97-pass-undo/final-results.log`.
Only the six approved source/specification files changed. The other 685 repository files remain unchanged from the step baseline.
All 29 implementation boxes remain open. No commit, publication, live device access or owner-project edit occurred.

## Capture-aware autosave work in progress

The owner approved the six-file boundary for unsaved-state tracking and automatic saving of complete pattern takes.
Its contract is in [the eighth implementation boundary](design.md#eighth-implementation-boundary-capture-aware-autosave).
The collector now reports changed raw rows, completed Gates and changed completed passes to the existing pending-save state.
Default A1 creation also marks unsaved work. Repeated collection keeps the original pending-save timestamp.
Empty passes and runtime-only descriptors do not mark unchanged existing A1.

Autosave first collects accepted capture, then defers through requested start, CountIn, Recording and live Finishing.
It waits for accepted Idle, even when model rows are complete but an acknowledgement remains queued.
After audio loss, complete recovered rows can save without acknowledgement from the stopped instance.
Interrupted count-in and an unaccepted start do not leave permanent autosave deferral.
The shared save-eligibility helper, exit policy and manual read/write methods remain byte-identical to the step baseline.

The first new test reproduced the missing unsaved state:

```text
assertion `left == right` failed
  left: Clean
 right: Unsaved
```

All 13 new tests passed. They use disposable scratch sessions and projects, the production collector, and real stored-file reads.
The main Gate test crosses keyboard input, raw MIDI notes and learned pad buttons.
Checks cover exact final duration 240 ticks, format 5, pending status, queued start/acknowledgement, interruption, Undo and project reset.
Read/write cases cover damaged/newer files, an occupied temporary file, missing project folder and the existing successful manual retry after Idle.
The tests check that autosave does not recreate a missing project folder or clear a prior refusal.

Commands and literal results:

```text
cargo test -p ptsr --bin ptsr pattern_autosave_ -- --nocapture
  test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 866 filtered out; finished in 1.97s
cargo test -p ptsr --bin ptsr app_capture_ -- --nocapture
  test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 865 filtered out; finished in 4.30s
cargo test -p ptsr --bin ptsr pass_undo -- --nocapture
  test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 866 filtered out; finished in 2.56s
cargo test -p ptsr --bin ptsr -- keyboard_and_controller_save_feedback_agree save_refusals_keep_priority_and_state_until_a_real_retry_succeeds --nocapture
  test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 877 filtered out; finished in 0.10s
```

The required `task check` exited 201. Names, format, licenses and Clippy passed.
All 879 App tests passed in 152.58 seconds, including the known shared-keymap tests.
That success does not repair their existing shared-directory race. This boundary does not change those helpers or callers.
The required run then reached the scenario audit, which still reports 56 uncovered scenarios.
The new autosave test covers one previously uncovered scenario. No scenario check changed or skipped.

```text
active scenarios need a named test or an attached scenario description:
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.55s
error: test failed, to rerun pass `-p ptsr --test scenarios`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

Separate engine suites passed with totals 27, 11, 8, 10, 11, 21, 16 and 18, including callback allocation/deallocation checks.
The TUI suite passed 132 tests. All three workspace documentation suites passed with zero tests.
Strict OpenSpec validation passed all 18 items. The existing change-96 archive prerequisite remains informational.
Neither change entered the archive.
The main-folder `task build` passed in 19.20 seconds. Final format, scope and text checks remain in the local report.

Separate `task bench` exited 0. Steady wall p99/max were 16.21/38.08%.
Dense-pattern wall p99/max were 7.60/15.63%.
Both workloads used 20000 callbacks and reported zero blocks above 50%.
Dense playback used 4096 hits and 32 voices with zero dropped hits.
This result does not explain earlier over-budget maxima or measure active capture, publication, erasure, Undo or autosave.

The complete commands, exits and logs are in `target/97-autosave/final-results.log`.
Only the six approved files changed. The other 685 repository files remain unchanged from the step baseline.
All six existing change-97 WAV files remain byte-identical. This boundary adds no WAV.
No commit, publication, live device access or owner-project edit occurred. Physical-controller acceptance remains unverified.
Manual save during recording, exit finalization, project-picker guards, erasure and active/global Undo remain outside this boundary.
Recording controls remain disabled. The required check still fails and all 29 implementation boxes remain open.

## Manual-save completion work in progress

The owner approved eight files for the manual-save part of task 6.1, then approved `crates/app/src/notices.rs` as the ninth file.
The contract is in [the ninth implementation boundary](design.md#ninth-implementation-boundary-manual-save-completion).
First collect accepted capture, then cancel count-in or finish the matching pass without blocking input.
Preserve one request, original input source and project ownership through delayed completion.
Use the ordinary writer and five-visible-second success notice only after the complete boundary.
Keep the completed take, Undo and unsaved status on refusal. Preserve existing manual retry and file protection.

Test production keyboard, learned note and CC routes, pending Gates, count-in, saturated queues, repeated saves and loss of audio.
Also test autosave priority, ownership reset, exact stored format 5 and existing Idle save behavior.
Exit finalization, project-picker entry guards, erasure, active/global Undo and recording activation remain outside this boundary.
The engine, saved format and dependencies stay unchanged. This boundary adds no WAV.
All 29 implementation boxes remain open. The required check still fails its scenario audit.

### Manual-save notice correction and verification

The first test reproduced success before capture completion. Before the ninth file, the focused suite passed eight tests and failed one expiry test:

```text
cargo test -p ptsr --bin ptsr pattern_manual_save_ -- --nocapture
assertion `left == right` failed
  left: Some("Finishing A1 before saving.")
 right: None
test result: FAILED. 8 passed; 1 failed; 0 ignored; 0 measured; 879 filtered out; finished in 3.13s
```

The ordinary queue correctly retains interrupted notices. The owner approved a ninth file to cancel only progress owned by the pending save.
The request now keeps the existing entry identity. Completion, refusal and project reset remove only the matching entry.
No queue clearing, artificial clock advance or test weakening occurred. Notice history and unrelated errors remain under existing policies.
Stale, expired, replaced and evicted entries cannot remove a newer entry, even with equal text and origin.
The repeated-input test now completes all three input routes. The tests also cover progress hidden behind Warn/Error and reset behind a warning.

Verification commands and literal output are in local `target/97-manual-save/results.log` and its source logs.
The first `task check` stopped at Clippy because the new reopen test cloned a comparison value.
The correction compares slices without cloning. The repeat run passes names, format, licenses and Clippy.
All 892 App tests and 24 notice integration tests pass, including the notice identity tests in both harnesses.
The required run then fails its scenario audit: 53 active scenarios remain without a matching test, down from 56 before this boundary.
Its literal result is `2 passed; 1 failed` and Task exits 201. The required run does not reach its benchmark.
Do not treat separate passing checks as a successful required check.

| Check | Result |
| --- | --- |
| `cargo test -p ptsr --bin ptsr pattern_manual_save_ -- --nocapture` | 13 passed |
| `cargo test -p ptsr --bin ptsr pattern_autosave_ -- --nocapture` | 13 passed |
| `cargo test -p ptsr --bin ptsr app_capture_ -- --nocapture` | 14 passed |
| `cargo test -p ptsr --bin ptsr pass_undo -- --nocapture` | 13 passed |
| `cargo test -p ptsr --bin ptsr saving_tests:: -- --nocapture` | 10 passed |
| Existing keyboard/controller save feedback and refusal tests | 2 passed |
| Separate engine capture, recovery, count-in, playback and callback allocation tests | 27/11/8/10/11/21/16/18 passed |
| `cargo test -p tui` | 132 passed |
| `cargo test --workspace --doc` | Three suites passed with zero tests |
| `task openspec:check` | 18 passed, 0 failed |
| Main-folder `task build` | Release build passed in 16.23 seconds |

Separate `task bench` exits 0, but dense-pattern wall maximum reaches 58.90 percent against the 50 percent target.
One of 20,000 dense callbacks exceeds that target. Thread CPU maximum is 24.32 percent, with zero dropped hits and 32 voices.
Steady-playback wall p99/maximum is 14.92/19.17 percent. A successful p99 exit does not establish the maximum-time requirement.
The cause remains unknown. Active recording and manual-save callback cost remain unmeasured.
The unchanged shared temporary-keymap race remains unfixed despite its tests passing in this run.

Nine approved files changed. The other 682 tracked or unignored files remain unchanged against the manual-save baseline.
The six existing change-97 WAV files remain byte-identical. This boundary adds no WAV or engine change.
The original save eligibility, exit saving, serializer and read/write protection remain byte-identical.
All 29 implementation boxes stay open. No commit, staging, publication, live-device action or owner-project write occurred.
Recording controls remain disabled. Exit, picker entry, audio-instance replacement and live acceptance remain outside this boundary.

## Quit and project guards work in progress

The owner approved continuation through the existing change-97 plans without further per-boundary confirmation.
This tenth boundary covers the quit and project-transition parts of tasks 2.2 and 6.1.
The contract is in [the tenth implementation boundary](design.md#tenth-implementation-boundary-quit-and-project-guards).
Freeze quit at the last coherent accepted boundary. Close recoverable Gates without a new callback, publication or acknowledgement.
Retry concurrent snapshot or terminal reads on normal UI turns. Do not classify a normal quit as backend loss.
Keep the original pending manual save and its input source. Do not retry an earlier failed write on exit.
Refuse project selection, open and creation before any save or filesystem side effect during an active or requested pass.
The ninth file only clarifies the existing engine recovery documentation for a closed prefix. Engine executable logic stays unchanged.

Ten focused `pattern_lifecycle_` tests pass. They use production keyboard and raw MIDI paths with disposable projects.
They cover held Gates, count-in, unaccepted starts, full queues, real audio loss, delayed events, save ownership and file protection.
They also cover live concurrent Gate completion, an unavailable live snapshot and Idle project transition behavior.
The unavailable-snapshot test first failed because the collector discarded a queued start. The collector now retains that start until a coherent boundary.
The concurrent read path does not spin, sleep or change the backend flag.
Recording activation, erasure, active/global Undo and remaining display work follow this boundary.
All 29 implementation boxes remain open. The owner did not authorize commits, publication, device access or owner-project writes.

### Quit and project guard verification

Commands, exit codes and literal output are in local `target/97-lifecycle/results.log` and its source logs.
The focused lifecycle suite passes ten tests. Manual save, autosave, capture, pass Undo and existing save-policy regressions pass.
The required `task check` passes names, format, licenses, Clippy, 902 App tests and 24 notice integration tests.
It then fails the scenario audit with 51 omissions, down from 53. Task exits 201 before its benchmark.
The unchanged temporary-keymap race does not fail this run. It remains unfixed.
The first OpenSpec check rejects a requirement longer than 500 characters. The correction separates the quit requirement and shortens the project guard.
The repeat `task openspec:check` passes all 18 items without a requirement warning.

| Check | Result |
| --- | --- |
| `cargo test -p ptsr --bin ptsr pattern_lifecycle_ -- --nocapture` | 10 passed |
| `cargo test -p ptsr --bin ptsr pattern_manual_save_ -- --nocapture` | 13 passed |
| `cargo test -p ptsr --bin ptsr pattern_autosave_ -- --nocapture` | 13 passed |
| `cargo test -p ptsr --bin ptsr app_capture_ -- --nocapture` | 14 passed |
| `cargo test -p ptsr --bin ptsr pass_undo -- --nocapture` | 13 passed |
| `cargo test -p ptsr --bin ptsr saving_tests:: -- --nocapture` | 10 passed |
| Separate engine capture, recovery, count-in, playback and allocation suites | 27/11/8/10/11/21/16/18 passed |
| `cargo test -p tui` | 132 passed |
| `cargo test --workspace --doc` | Three suites passed with zero tests |
| Main-folder `task build` | Release build passed in 16.45 seconds |

Separate `task bench` exits 0. Dense-pattern wall maximum is 75.22 percent, with one of 20,000 callbacks above 50 percent.
Dense thread CPU maximum is 23.24 percent, with 32 voices and zero dropped hits. The maximum-time requirement remains unmet.
Steady-playback wall p99/maximum is 14.93/29.97 percent. Active capture and local quit recovery performance remain unmeasured.
Nine scope files changed. The other 682 files and all six change-97 WAV files remain byte-identical to the lifecycle baseline.
The engine change contains documentation only. Save eligibility, serialization and write protection remain byte-identical.
The required check remains red. Remaining recording controls, erasure, display work and live acceptance are not complete.

## Accepted engine erasure work in progress

The eleventh boundary covers the engine part of task 5.1 before recording activation.
The contract is in [accepted engine erasure](design.md#eleventh-implementation-boundary-accepted-engine-erasure).
Retain bounded accepted intervals and removed identities. Suppress every matching collision before its sound can start.
Preserve prior ordinary holds, exact release endpoints, loop seams and musical one-loop expiry.
Recycle completed Gate slots only through matching explicit reclamation, so row removal can make room safely.
Test pressure, stale consumption, project changes, live grid changes and more than 4096 accepted onsets in one pass.
Add one generated six-second erasure render with independent numerical tests.
App interval application, fresh in-pass row identities, input activation and full pass Undo remain integration work.
All 29 implementation boxes remain open. Required `task check` remains red until substantive remaining scenarios pass.

### Accepted engine erasure verification

The engine suites pass: 30 library, 11 capture, 8 Gate, 10 recovery, 11 count-in, 29 erasure, 21 playback, 16 recorded-playback and 22 allocation tests.
The erasure tests include 36 sample-rate, tempo and block-size combinations, concurrent recovery readers, queue pressure and more than 4096 captured onsets.
The new allocation tests cover all 4096 removed rows, full interval and retirement storage, 32 voices, and 5000 onsets with reusable Gate slots.
They also test 16 fading slots and stale captured publications through real retirement and project queues.
The full required run includes the App lifecycle, manual save, autosave, capture and Undo tests.
TUI tests pass 132 tests. Workspace documentation suites pass with zero tests.

An existing project-reset test first detects a nonzero frame-duration field in Idle: actual 0.04, expected 0.0.
The correction keeps Idle and CountIn empty and freezes terminal frame duration at pass end. The existing assertion stays unchanged.
The complete engine repeat passes. Existing assertions remain active. No tests skip.

Review reproduced three timing failures before their corrections.
They affect the last matching onset at expiry, manual release in the same frame, and early rounded expiry.
Tests also reproduced a retirement panic when a sequenced pad inherited the silent manual-target exception.
Separate scheduling windows now outlive UI feedback consumption. Empty windows release their space at once.
Closed windows keep their original grid and strength. Current-prefix publications apply later audio progress before playback.
Pass and sequence checks reject stale publications after window expiry and preserve a later onset in the same output frame.
Terminal capture includes the last accepted automatic-expiry row before it freezes. Later callbacks cannot extend this result.

Required `task check` exits 201. Names, format, licenses and Clippy pass, then 902 App tests and 24 notice tests pass.
The scenario audit still reports 51 uncovered scenarios:

```text
active scenarios need a named test or an attached scenario description:
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.54s
error: test failed, to rerun pass `-p ptsr --test scenarios`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

Separate `task bench` exits 0, but its paired timing exceeds the required maximum.
Steady wall p99/max is 15.30/37.99 percent. Dense wall p99/max is 8.42/84.83 percent, with one block above 50 percent in 20000.
Dense thread CPU maximum is 25.34 percent. Dense playback has 4096 hits, 32 voices and zero dropped hits.
The local trace records 2262048 ns wall time and 292929 ns thread CPU time for the overrun.
Its wider measurement envelope records CPU IDs 2 then 4 and two involuntary context switches.
These observations do not establish the exact cause or satisfy the callback maximum. Active erasure performance remains unmeasured.
OpenSpec validation passes 18 items. Main-folder `task build` passes in 26.99 seconds.

`renders/97-pattern-erase.wav` has 288000 frames, six seconds, stereo float32 at 48000 Hz.
Both channels match an independently compiled reference within 0.000001 per sample for blocks of 1, 128 and 1024 frames.
Separate analytic errors are 0.000000296432 and 0.000000468509. Finite samples, clipping, levels, one-frame timing, cuts and final silence pass.
Owner listening and physical-controller acceptance remain unverified.

Detailed commands, literal results and retained failures are in `target/97-erase-engine/results.log`.
Ten scope files changed. The other 682 files and all six earlier change-97 WAV files remain byte-identical to this boundary's baseline.
Recording controls remain disabled. App row removal, fresh in-pass row generations and complete erasure Undo remain integration work.
All 29 implementation boxes remain open. No commit, publication, live device access or owner-project edit occurred.

## App erasure integration work in progress

The App now applies accepted engine erasure to stored raw rows before input activation.
Its six-file scope and edge cases are in [App erasure and reusable rows](design.md#twelfth-implementation-boundary-app-erasure-and-reusable-rows).
Reusable row slots keep increasing generations across Undo. Pending Gates retain row identities instead of shifting vector indices.
Compilation uses each retained interval's grid and strength. Publication precedes metadata consumption, reservation confirmation and final acknowledgement.
Compile failure retains work. Repeated UI polls cannot erase restored Undo rows or send duplicate prefix confirmations.
Save, quit and audio loss retain the complete owned prefix. Saved format 5 and four-integer rows stay unchanged.

The 12 new App tests pass, including 4097 accepted onsets, full raw capacity, stale Gate consumption and queued Undo.
The grid matrix covers nine grids, strengths 0, 50 and 100, and three production pad input sources.
An early rounded expiry preserves a later onset in the same output frame.
A dedicated concurrent App erasure-reader test remains open. Lower-level coherent-read tests do not replace that App test.

The required `task check` exits 201. All 914 App tests and 24 notice tests pass, then the scenario audit reports 50 uncovered cases.
The full engine suite passes separately, including 31 library, 29 erasure and 22 allocation tests.
The TUI suite passes 132 tests. Documentation tests pass. OpenSpec validation passes all 18 items.
Main-folder `task build` passes in 19.69 seconds.
The separate benchmark exits zero. Steady wall maximum is 35.88 percent and dense wall maximum is 16.23 percent.
Neither workload exceeds 50 percent in 20000 callbacks. This does not explain historical overruns or measure active erasure.

The new local `renders/97-pattern-app-erasure.wav` lasts six seconds, with stereo float32 samples at 48000 Hz.
Independent float32 source sums match both channels exactly. All 22 starts stay within one frame, with finite samples and required silence.
A double-precision sine probe first exceeded 0.000001 on the right channel.
A source-only check attributes that difference to float32 phase calculation, not playback. The playback tolerance stays at 0.000001.
Listen as follows:

1. Listen to the low kick and the unchanged high-tone rhythm.
2. Check the missing low kicks at 0.5 and 2.5 seconds.
3. Check the restored low kick at 4.5 seconds after Undo.
4. Check the transitions for clicks.

Owner listening and physical-controller acceptance remain unverified.
Detailed commands, literal results and retained failures stay in `target/97-app-erasure/results.log`.
The other 686 files and all seven earlier change-97 WAV files remain byte-identical to this boundary's baseline.
All 29 implementation boxes remain open. Keyboard/MIDI activation, active Undo, the controls panel and indications remain later work.
No commit, publication, live device access or owner-project edit occurred.

## Erasure-reader and global Undo work in progress

The owner approved the remaining change-97 plans without another confirmation for each existing implementation boundary.
The two new boundaries are [erasure-reader completion](design.md#thirteenth-implementation-boundary-erasure-reader-completion) and [global Undo](design.md#fourteenth-implementation-boundary-global-and-active-pass-undo).
This remains partial implementation, not a usable recording trial.

Two App tests check failed bounded erasure reads and concurrent engine interval completion.
A failed read defers consumption, row confirmation and pass acknowledgement until the full accepted prefix is available.
Stable retry retains unrelated rows and produces one completed-pass step.

GLOBAL `pattern.undo` now uses Alt+Z. F1 describes pass completion and the absence of redo.
The Alt modifier line uses the short label `pattern undo`.
Active Undo finishes the matching pass, collects its accepted prefix and restores only pre-pass pattern rows and settings.
Restoration follows final publication and acknowledgement. Autosave waits until restoration completes.
Count-in cancellation and an empty pass preserve older history. A stopped backend needs no completion command.
Idle pattern Undo refuses a newer common pad or FX edit instead of changing history out of order.
Local Chop Undo remains separate. Pads Ctrl+Z cancels an active pass before it can reach older history.

Six App tests use production keyboard, raw note and learned CC dispatch.
They cover all five screens, held Gates, erasure, count-in, pending starts, audio loss, newer swaps and a full 1024-command queue.
They also check local Chop history, text ownership, project restoration and a later pass after cancellation.
One input test checks both layouts, uppercase input, repeat, release, explicit removal and owner remapping.
Questions and learning still need their dedicated pattern-command checks in the remaining integration tasks.

The first compile identified two exhaustive help lists that needed the new action.
An initial screen fixture created a newer shared pad edit when it reopened a changed linked slice.
The production Undo guard correctly refused that edit. The final fixture asserts a latest pattern step and separates local Chop history.
The first required check found an old exact Alt-help expectation without Z.
The short label and exact expected list now include Z. The focused Alt-help test passes without a weaker assertion.
Earlier failures remain in the local logs.

Verification:

- The two focused erasure-reader tests pass. The six focused global Undo tests pass.
- The focused input and Alt-help tests each pass one test.
- Final `task check` passes names, format, licenses, Clippy, 922 App tests and 83 input tests.
  The active-scenario audit still fails with 49 uncovered scenarios, down from 50. No audit rule or scenario name changed.
- Separate `task screens` passes six tests. The existing catalogue contains 354 ANSI files and 192 additional Pitch Gate frames.
  These checks do not establish the missing change-97 controls or phase indications.
- Separate `cargo test -p tui` passes 132 tests. Documentation tests pass with zero tests in each documentation suite.
- `task openspec:check` passes all 18 items. Its archive-order information does not authorize archive or publication.
- Final main-folder `task build` passes in 15.56 seconds.
- Separate `task bench` exits zero, but one of 20000 steady callbacks reaches 61.07 percent of the buffer budget.
  Steady wall p99 is 13.33 percent. Dense wall p99/max is 6.45/31.06 percent, with zero dropped hits.
  The 50 percent maximum requirement remains unmet in this measurement. The cause is unknown. Active recording and erasure remain unmeasured.
- New English prose uses STE-flavored review. Changed prose has no hard lint findings.
  Whole-design lint retains nine earlier hard findings outside these two new boundaries. This is not full dictionary compliance.

The final required check reports:

```text
test result: ok. 922 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 141.61s
```

It then reports:

```text
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.78s
error: test failed, to rerun pass `-p ptsr --test scenarios`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

The Task exit code is 201. Its benchmark stage does not run after the audit failure.
Detailed logs stay in `target/97-erasure-reader/` and `target/97-pattern-undo/`.
Recording-start bindings, the controls panel, new phase feedback, complete command guards and the final demonstration WAV remain unfinished.
These boundaries did not use live devices or owner projects. Dependencies and saved format stay unchanged.
No commit or publication occurred.
All implementation boxes remain open until required checks pass and authorized atomic commits exist.

## Recording commands and real Chop pads work in progress

The [fifteenth boundary](design.md#fifteenth-implementation-boundary-recording-commands-and-real-chop-pads) connects recording and erasure through the production input routes.
Ctrl+R addresses A1 on all five screens. Alt+E toggles recording erasure.
Both commands appear as GLOBAL buttons for MIDI learning. Explicit owner bindings retain priority.
Alt+R keeps microphone capture. Ctrl+T keeps output capture. Ctrl+R no longer shows the old microphone move notice.

A pending start owns its pass before a callback accepts it.
Another press ends that pass. Repeated finish requests cannot create another pass.
Erasure retains command intent so two toggles before a callback still mean On then Off.
Accepted audio state remains authoritative for phase. Neither command claims success from a redraw.
A full command queue retains the existing accepted pending-start behavior and autosave protection.

Microphone or output capture refuses pattern start before A1 creation.
Pattern count-in, recording, finishing and pending completion refuse audio capture without changing targets.
A typed recording-refusal notice temporarily precedes the audio-capture duration line.
Other notices retain their recording precedence. An earlier Error keeps priority and its normal acknowledgement.
The refusal uses six seconds of visible time. Hidden Info time remains available after it expires.
Placement, marking and Chop assignment cannot consume recording pads.
Questions keep priority over the placement guard. Earlier permitted releases still finish their original Gates.
An unclaimed Esc ends recording or cancels count-in after existing overlays close.

Chop uses real pad assignments during capture, including empty pads and absolute bank addresses.
The packed MIDI focus includes that route without changing the saved project format.
Entering Chop during capture does not hush voices or change banks to another slice bank.
Linked manual hits still select their matching slices. Space and cursor previews do not enter A1.
After completion, an unassigned Chop returns to its slice keyboard.
A keyboard without releases targets erasure for one loop. Ordinary one-shot playback stays unchanged.

Fifteen new App tests cover production keyboard, raw notes and learned CC controls.
One new input test checks recording and erasure keys in both layouts, repeat, release and owner overrides.
A notice unit test checks the typed refusal behind a hidden Error, its expiry and replacement by identical ordinary text.
The focus test now checks all 2048 mode combinations across every screen and bank.
Two explicit snapshots and the reserved-bit test require mechanical updates for the additional focus bit.
Partial scenario coverage does not receive a complete-scenario annotation.
The controls panel and the combined raw erasure burst still need their dedicated scenario checks.

Verification and before-file copies stay in `target/97-record-controls/`. The final commands use the `-verified` log suffix.

- `task check` passes names, format, licenses, Clippy and all executed ordinary tests.
  App reports `938 passed; 0 failed`, input `84 passed; 0 failed` and notices `25 passed; 0 failed`.
  The unchanged active-scenario audit still fails with 38 uncovered scenarios. Task exits with 201.
- `task build` reports ``Finished `release` profile [optimized + debuginfo] target(s) in 15.36s``.
- `task openspec:check` reports `Totals: 18 passed, 0 failed (18 items)`.
  Its existing informational archive warning remains. This change is not archived.
- `task screens` reports `6 passed; 0 failed` and retains 354 catalogue files plus 192 Pitch Gate frames.
  It does not establish the unfinished recording-state catalogue coverage.
- `cargo test -p tui` reports `132 passed; 0 failed`.
- `cargo test --workspace --doc` passes three documentation suites, each with zero tests.
- The changed boundary prose uses STE-flavored review. Its lint reports zero findings, not full dictionary compliance.
- The required audit failure prevents the automatic benchmark. Active recording and erasure remain unmeasured.
  The earlier callback-budget failure remains unresolved. The complete `renders/97-pattern-record.wav` is still absent.

The required failure remains visible:

```text
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.72s
error: test failed, to rerun pass `-p ptsr --test scenarios`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

Earlier diagnostic failures remain available: the first visibility check failed the stronger refusal assertion.
A later Clippy check rejected notice methods unused by the standalone notice suite.
The additional notice unit test exercises those methods without disabling dead-code checks.
The first compile exposed explicit test snapshots without the additional focus field.
An initial regression run exposed stale Finishing routing after accepted Idle feedback.
The route now follows accepted Idle without waiting for another collection turn.
The old autosave test also established that a full command queue accepts a pending start.
This boundary preserves that contract instead of refusing the accepted request.
A raw-input fixture replaced its note binding when it learned a CC for the same pad.
The corrected fixture keeps the note route for note tests and the CC route for CC tests.
The first refusal test inspected history but did not prove that the owner could see the message.
The duration line and notice visibility both hid the refusal during audio capture.
The stronger production test checks the message that the frame shows. A controlled-time test checks priority and remaining notice time.
The notice model uses a typed marker instead of classifying text. It does not expose unrelated notices during recording.
Earlier failures remain in the logs. No assertion skip or audit change was necessary.

This is not the final owner trial. Compact controls, complete phase feedback, the final WAV and active capture performance checks remain unfinished.
No live device, owner project, dependency, commit, publication, archive or merge operation occurred.
All implementation checkboxes remain open.

## Absolute pattern parameters work in progress

The [sixteenth boundary](design.md#sixteenth-implementation-boundary-absolute-pattern-parameters) connects GLOBAL knobs `pattern.bars`, `pattern.quantize` and `pattern.strength`.
They use the existing profiles, input dispatch, takeover responses and applied-value popup.
The compact Alt+N panel and its navigation buttons are not implemented by this boundary.

Bars uses 1–64, Quantize uses all nine specified grids and Strength uses 0–100 in steps of ten.
A raw value of 64 selects 33 bars, 1/32 or 50 percent. Values 0 and 127 reach the two limits.
Raw timing, row order and hidden rows remain unchanged.
Pickup waiting cannot create A1 or show an unapplied value. Accepted input can create the existing empty default A1.
Pending starts, CountIn, Finishing and pending end/Stop refuse edits. Recording refuses Bars but permits grid and strength.
A queued Stop now retains ending intent before callback feedback without changing the accepted phase indicator.

Recording publications retain the collector's accepted pass and sequence.
A full queue can make a publication stale while erasure continues. The next collection retries the current grid without restoring erased rows.
Tests inspect silence at the old grid position and actual sound at the new position, not only the saved grid string.
Completed and active pass Undo reset parameter takeover after restoration. Project replacement also resets the targets.
Offline edits after audio loss wait for local capture recovery, then avoid commands to the stopped callback.
This boundary adds no Idle-setting Undo category.

Twelve new tests pass through raw MIDI or the explicit invalid-value boundary.
Range checks use 0, 64 and 127 on all five screens. A separate raw-MIDI check covers all 128 positions on Pads.
The tests also cover raw-row preservation, two-device pickup and scale, Undo, full queues, held erasure targets, blocked input and audio loss.
The external-setting test exercises takeover reset directly. It does not establish the later keyboard-panel route.
The first phase test mixed the start's earlier publication with the knob result.
The fixture now drains that earlier frame before checking commands from the refused knob input.
The failed run remains in `target/97-pattern-parameters/edges.log`.

Review now treats the earlier no-release erasure test as partial scenario coverage.
That test proves one-loop expiry. The fallback message and ordinary one-shot checks remain unfinished.
The new absolute-range test covers its complete named scenario. Other partial tests do not gain complete-scenario annotations.
The audit and specification scenario names remain unchanged.

Verification stays in `target/97-pattern-parameters/`:

- `cargo test -p ptsr --bin ptsr pattern_parameters_ -- --nocapture` reports `12 passed; 0 failed` in `edges-3.log`.
- `task check` passes names, format, licenses, Clippy and all executed ordinary tests.
  App reports `950 passed; 0 failed`, input `84 passed; 0 failed` and notices `25 passed; 0 failed`.
  The six screen-catalogue tests pass in that workspace run. The separate `task screens` command was not run for this boundary.
- The required active audit still reports 38 uncovered scenarios and Task exits with 201.
  One complete knob-range scenario replaces the premature no-release annotation in the coverage count.
- `task build` reports ``Finished `release` profile [optimized + debuginfo] target(s) in 20.71s``.
- `task openspec:check` reports `Totals: 18 passed, 0 failed (18 items)` with the existing informational archive warning.
- `cargo test -p tui` reports `132 passed; 0 failed`. `cargo test --workspace --doc` passes three zero-test suites.
- Changed prose uses STE-flavored review. Its lint result does not establish full dictionary compliance.
- The audit stops `task check` before its benchmark. Active recording and erasure performance remain unmeasured.

The required failure is:

```text
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.66s
error: test failed, to rerun pass `-p ptsr --test scenarios`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

All implementation checkboxes remain open. The final recording WAV and active capture performance checks remain unfinished.
No live device, owner project, dependency, commit, publication, archive or merge operation occurred.

## Compact pattern controls work in progress

The [seventeenth boundary](design.md#seventeenth-implementation-boundary-compact-pattern-controls) connects Alt+N and five GLOBAL control buttons.
Bars, Quantize and Strength use the existing accepted-setting path from the previous boundary.
The panel retains the underlying screen, bank, selected pad and edit state.
Keyboard directions use the existing edit layer, including explicit owner overrides and `none`.
MIDI parameter and value buttons require the panel. Named parameter knobs remain available without it.
Opening without A1 creates the existing default. Phase restrictions still govern setting edits.
Changed button values reset their takeover target. Pickup waiting cannot claim an unapplied change.

The shared drawing uses seven rows in the right column, not a centered dialog over the pads.
The live frame includes its area in the terminal graphics blockers.
Questions, text, pickers, learning and full help hide the panel without changing their input ownership.
Esc closes learning before the panel. An unclaimed Esc closes the panel before ending recording.
Screen and project transitions close the panel. No saved format or callback change is necessary.

Ten focused tests pass. They exercise keyboard and raw note or CC routes, default creation, length limits and hidden rows.
The tests also check playable Gate pads, original edit state, both layouts and keyboard protocols, overrides and two-device takeover.
All eight pattern commands pass confirmation, cancellation, clearing and restart checks through the existing F12 MIDI path.
After restart, raw buttons execute on every current screen without changing existing X-Session bindings.
A catalogue test checks each selected row at both sizes, with hidden pads and safety warnings.
Every cell outside the panel matches the same underlying view without the overlay drawing.
The test checks the fixed rows of both closed and open frames separately.
Chop intentionally changes selected-pad and dimming styles when controls activate real-pad routing.
Comparing those styles to the closed slice-preview role was not a valid overlay reference.
The production-route test separately checks real pads, bank changes and original Gate releases after closing controls.
The regular catalogue now requires the controls entry on every screen.

Evidence stays in `target/97-pattern-panel/`. Required verification continues in `target/97-bound-zoom/`.
The first full check found an outdated Alt-help expectation. Its exact test passes after the correction.
The later zoom check found a short-help regression. The zoom section below records that failure.
The initial test compile used a stereo result as an iterator. The assertions now inspect both channel vectors.
Two first-run fixtures used an incorrect initial bar count and changed raw rows without their runtime identities.
The fixtures load valid session state before exercising production input.
The initial implementation exposed private capture fields to the drawing module. A read-only restriction method keeps those fields private.
Review moved the panel's Esc guard after learning and full-help guards. A production test checks that priority.
All assertions remain enabled. All implementation checkboxes remain open.
Full accepted phase feedback, erasure markers, the final WAV and active recording performance checks remain unfinished.
No live device, owner project, dependency, commit, publication, archive or merge operation occurred.

## Accepted recording feedback work in progress

The eighteenth boundary uses one accepted audio snapshot per frame for badges, erase targets and phase help.
It adds an Erase corner and catalogue fixtures from the real offline engine.
Pending commands do not claim accepted phases. Four count digits, tick zero and the next beat follow audio time.
Five focused tests pass through keyboard, raw notes, learned CC commands and shared drawing.
The catalogue test checks 160 phase and overlay frames plus ten offline safety frames at both sizes.
Silent targets keep independent releases across banks. Legacy keyboard erasure expires after one loop and returns to one-shot sound.
A replaced project rejects earlier recording feedback. No physical input or listening check took place.

The first feedback tests found a beat-rounding error at the exact boundary.
The badge now uses the existing clock's rounding tolerance and wraps within the pattern length.
A separate fixture incorrectly reused one CC for two simultaneous pad holds.
The test now uses independent learned CC 80 and 81. Release assertions remain enabled.
The catalogue comparison keeps hidden/edit roles equal and checks full cells outside the actual panel area.
It does not compare a hidden-grid meter against the different shown-grid layout.
The first full check found three catalogue regressions. The independent required-name list now includes all five new modes.
The title test must preserve an accepted recording engine, not replace count-in with ordinary playback.
A later model probe shows that controls alone create raw A1 data without an audio engine.
The title fixture now preserves only accepted active phases. Other modes retain the existing playback setup.
The original title, safety and coverage assertions remain enabled.

The F1 data contains settings, erase identities, save completion and separate no-release fallbacks.
Its existing renderer does not wrap or scroll the key table. Some rows therefore remain inaccessible.
Do not treat model-only assertions as completed F1 visibility scenarios.
The design records a proposed page-navigation change as an open owner decision.
Keep current navigation unchanged until approval. Evidence stays in `target/97-recording-feedback/`.
The captured `f1-clipped-80x26.ansi` shows that recording instructions do not appear in the visible key table.

Verification of the current partial implementation:

- Five focused feedback tests pass: `5 passed; 0 failed` in 61.34 seconds.
- `task check` passes names, format, licenses, Clippy and all executed ordinary suites.
  App reports `969 passed; 0 failed`, input `85 passed; 0 failed` and notices `25 passed; 0 failed`.
- The active audit lists 25 uncovered scenarios, down from 31. It reports `2 passed; 1 failed` in 9.66 seconds.
  Task exits with 201. The benchmark stage does not run. The audit rules remain unchanged.
- `task screens` reports `8 passed; 0 failed` in 112.85 seconds.
  It generates 414 catalogue frames, 192 additional Pitch Gate frames and 170 additional recording frames.
- Main-folder `task build` reports ``Finished `release` profile [optimized + debuginfo] target(s) in 0.08s``.
- `task openspec:check` reports `Totals: 18 passed, 0 failed (18 items)` with the existing informational archive warning.
- `cargo test -p tui` reports `132 passed; 0 failed`. `cargo test --workspace --doc` passes three zero-test suites.
- `git diff --check` passes. Local Markdown files and anchors pass their checks.
  Changed English prose uses STE-flavored structural review, not full dictionary compliance.

The full results and retained diagnostics stay in `target/97-recording-feedback/results.log`.
The final recording WAV, active capture performance, long-name and edit-layer help checks remain unfinished.
All task checkboxes remain open. No commit, publication or owner trial is complete.

## Zoom while editing slice bounds work in progress

The owner approved Ctrl/Alt+I/K zoom on Chop without releasing the bound modifier.
The [zoom boundary](design.md#zoom-while-editing-slice-bounds) adds four aliases to existing zoom actions.
Ctrl+Up/Down keeps tempo. Explicit owner overrides and `none` keep priority.
Legacy Tab keeps its action. Alt+I replaces an indistinguishable Ctrl+I in legacy terminals.

Four focused App tests pass through the production keyboard handler.
They cover both layouts, repeat and release, both bounds and open or closed slice edit.
They check effective Ctrl, Alt and F1 help, owner overrides, assignment, naming and learning guards.
A separate legacy sequence uses Alt+I/K and keeps Tab distinct from an unavailable Ctrl+I event.
The input suite passes all 85 tests, including aliases, layouts, owner removals and unchanged tempo chords.

The first full zoom check failed two existing help tests: `961 passed; 2 failed`.
Adding aliases to the short zoom row hid its plain keys under modifier filtering and made legacy help overflow.
The short row now keeps plain zoom keys. Modifier help and F1 retain the effective aliases.
Both existing failed help tests pass in exact reruns and in the full ordinary App suite.
The panel overlay comparison now passes independently with all ten focused panel tests.
This boundary changes no audio processing, controller assignment or saved format.
Final verification of this combined controls and zoom state:

- `task check` passes names, format, licenses and Clippy. App reports `964 passed; 0 failed`, input `85 passed; 0 failed` and notices `25 passed; 0 failed`.
- The active audit reports 31 uncovered recording scenarios. Its result is `2 passed; 1 failed` and Task exits with 201. The audit rules remain unchanged. The benchmark stage does not run.
- `task build` reports ``Finished `release` profile [optimized + debuginfo] target(s) in 19.12s``.
- `task openspec:check` reports `Totals: 18 passed, 0 failed (18 items)` with the existing informational archive warning.
- `task screens` reports `7 passed; 0 failed`, 364 catalogue ANSI files and 192 additional Pitch Gate frames.
- `cargo test -p tui` reports `132 passed; 0 failed`. `cargo test --workspace --doc` passes three zero-test suites.
- Four focused zoom tests and ten focused panel tests pass. No physical keyboard, controller, listening or live terminal check took place.
- Changed English prose uses STE-flavored review and structural lint. This review does not establish full dictionary compliance.

Evidence stays in `target/97-bound-zoom/results.log` and `target/97-pattern-panel/results.log`.
The final recording WAV and active recording or erasure performance remain unfinished.
All implementation checkboxes remain open. No commit or publication occurred.

## Approved erasure-release implementation boundary

The owner approved the ten-file extension and requested completion of recording and callback investigation.
The [accepted release protocol](design.md#accepted-release-protocol) specifies runtime command fields before implementation.
Keep ordinary manual releases unchanged. Retain the original project, physical release signal and resolved absolute pad.
Preserve both failing raw-input workflows. They must reach their stereo, three-loop and pass-Undo assertions before their scenarios close.
Check early releases, bank changes, repeated controls, independent ports and channels, disconnect, queue pressure and project replacement.
Count allocation and deallocation during acceptance, release, retirement pressure and stale-command discard.
Do not close any implementation checkbox or historical callback overrun from a partial passing run.
Preserve before-files, hashes, bounded diffs and literal results in `target/97-erase-release/`.
This implementation request does not authorize commits, publication, archiving or live-device actions.

## Proposed completion file boundary

The owner approved the [completion file boundary](design.md#proposed-completion-file-boundary) with an explicit go.
The release fix passes its focused checks, but 19 scenarios, the final WAV, owner procedures and active performance measurements remain unfinished.
The approved boundary adds nine files to the ten-file set. Keep changes within this set.
Preserve the current release evidence before the next boundary. Do not treat overlapping snapshots as independent commit patches.

## Approved sound-preserving filter work

The owner approved completion of the remaining plans.
The [filter calculation boundary](design.md#sound-preserving-filter-calculation-boundary) adds only the filter module and these two artifacts.
Preserve the existing glide and rendered sound, including its stalled residual state.
Cache coefficient inputs by their exact bits, including sample rate, without changing the old update conditions.
Add old-formula comparisons for every sample, integrators, target changes, type changes, default initialization and new voices.
Check stalled transitions at one and eleven seconds. Check cache reuse and invalidation independently.
Run `task check`, `taskset -c 12 task bench`, `task openspec:check` and `task build` without weaker workloads or limits.
Keep new local evidence separate from the previous completion and pause records.
Do not close historical maximum-budget failures from a microbenchmark or debugger run.
Commits, publication, archiving, merge, listening and physical-device checks still need their separate authorization or owner action.

The filter now retains coefficients for identical cutoff, damping and sample-rate bits.
Four new unit tests compare both output channels, coefficients, glide state and integrators with the previous formula.
The tests pass in debug and release builds. The one-second residual remains unchanged through eleven seconds.
Separate tests distinguish actual cache reuse from an identical recalculation and check all three input changes.
The default voice, restarted voice, mix-only changes and old skipped-update paths also pass.

Verification:

- `task check` exits zero. The ordinary suites pass 1516 tests, including 993 App tests and the four new filter tests.
  Names, format, licenses, Clippy, allocation checks and the unchanged scenario audit pass.
- `taskset -c 12 task bench` exits zero under the existing percentile checks for older workloads.
  Recording and erasure meet their maximum checks in this run. Dense playback and project switching still exceed the maximum target.
  A zero exit does not satisfy the maximum-budget requirement. Do not close the historical failures.
- `task openspec:check` reports `Totals: 18 passed, 0 failed (18 items)`.
  The existing change-96 archive prerequisite remains informational. This step archives neither change.
- `task build` completes the main-folder release binary.
- The boundary check finds exactly three approved source/specification files changed and 690 other files unchanged.
  All 46 existing WAV files remain identical, including `renders/97-pattern-record.wav`.
  Local links, Markdown anchors, fences, final newlines and `git diff --check` pass.
- Changed English prose uses STE-flavored structural review. This is not full ASD dictionary compliance.

Literal results, before/after hashes and the bounded diff stay under `target/97-perf-resume-m8wdmpjh/`.
The final render keeps the existing [owner procedure](../../../docs/LIVE-CHECKS.md#97-record-and-erase-a1).
This optimization adds no sound feature or new listening file.
A separate stack sample remains under `target/97-postfilter-stacks-J2RIdC/`.
It observes drive processing frequently during dense playback, without sampled coefficient calculations in that phase.
Debugger timings cannot establish deadline acceptance or the cause of a rare overrun.
Further sound-processing changes need a new file boundary and their own evidence before implementation.
All 29 implementation boxes remain open. No commit, publication, device access or owner-project edit occurred.

## Approved drive calculation work

The owner approved the three-file [drive boundary](design.md#approved-drive-calculation-boundary).
First test a local prototype without changing production processing.
Retain the existing transfer curve within 0.000001 per sample, normalization, zero bypass and parameter ramps.
If the prototype passes, change only `crates/engine/src/fx.rs` and these two artifacts.
Add dense numeric coverage and an independent stereo comparison through fixed and changing drive.
Keep existing sound assertions and add level, timing, silence and adjacent-step checks to the comparison render.
Run debug and release tests, the complete required check, CPU-12 workload, specification validation and normal release build.
Keep source snapshots and literal results in a new local evidence directory.
Do not close previous maximum failures from later successful samples or percentile exits.
No dependency, owner-project edit, device access, commit, publication or archive is part of this permission.

The prototype rejects the shorter eleven-term formula because its complete output exceeds the existing tolerance.
The accepted thirteen-term formula passes the same dense sweep and retains standard gain normalization.
Production output has a bounded rational calculation below magnitude nine and signed saturation above it.
Zero drive still bypasses exactly. Parameter ramps, input routing, effects and saved data retain their previous behavior.

Verification:

- `cargo test -p engine --lib fx::tests -- --nocapture` passes seven tests.
  The same seven tests pass with `--release`. Four tests are new, and existing assertions remain active.
  Coverage includes 2,000,001 shape inputs, 8,397,825 complete outputs, boundary neighbors and both stereo channels.
- `task check` exits zero. The ordinary suites pass 1520 tests, including 993 App tests and the unchanged scenario audit.
  Names, format, licenses, Clippy and 28 callback allocation tests pass.
- The separate `taskset -c 12 task bench` fails its maximum check.
  Both ordinary playback and erasure retain maximum violations. Do not substitute the successful full-check benchmark for this failure.
- A read-only kernel-counter observation accompanies another original CPU-12 benchmark run.
  That run has no maximum violations despite additional package-throttle events.
  An independent fixed-integer workload without the engine also shows variable CPU duration for identical work.
  These observations do not establish the cause of any earlier callback overrun.
  Make no third speculative sound edit without new failed-block evidence.
- `task openspec:check` reports `Totals: 18 passed, 0 failed (18 items)`.
  The existing change-96 archive prerequisite remains informational. This step archives neither change.
- `task build` completes the main-folder release binary.
- The source check finds exactly three approved files changed and 690 other files unchanged.
  Four existing WAV files change within the waveform tolerance. Independent checks cover both channels, level, timing, steps, clipping and silence.
  The previous left reference in the comparison WAV stays identical. The other 42 WAV files stay identical, including `97-pattern-record.wav`.

Literal results, prototypes, source snapshots, WAV comparisons and read-only observations stay under `target/97-drive-shppl00k/`.
The initial independent WAV reader expected a simple IEEE-float header and refused the actual extensible float format.
The reader now checks the extensible format's float GUID too. Waveform checks and tolerances remain unchanged.
Two bounded fetch attempts timed out without stderr. Remote freshness remains unverified.
The branch and HEAD remain unchanged. No commit, publication, archive, device access or owner-project edit occurred.
The [comparison listening procedure](design.md#drive-comparison-listening) remains an owner check, not an automated acceptance claim.
All 29 implementation boxes remain open. The callback maximum requirement remains unsatisfied.

## Approved temporary CPU-frequency diagnostic

The owner approved the [temporary CPU-frequency experiment](design.md#approved-temporary-cpu-frequency-experiment).
Prepare a local ordinary-user controller and fixed-path privileged guard before requesting a live run.
Test restoration and rejection paths with an in-memory policy, not the host sysfs files.
Keep the diagnostic workload, CPU affinity, sampling stride and maximum checks identical across A/B/A.
Preserve separate source snapshots, original policy values, command lines, output, counter brackets and exit codes.

The only permitted system write is the temporary CPU-12 minimum request and its restoration.
Keep its watchdog and recovery limitations explicit. Require private terminal authentication before any benchmark or policy change.
Do not execute the controller as root. Do not start workloads from the privileged guard.
Stop on policy interference or failed restoration. Keep partial evidence instead of silently restarting.
No production source, saved data, binding, sound or listening render changes in this boundary.
The maximum requirement remains open regardless of this diagnostic's result.
The local controller and guard pass 16 isolated tests without host sysfs writes.
The tests include a real closed pipe, a late guard, write acknowledgement loss and both restoration failure points.
They also check authentication failure before workload creation and refusal to run the controller as root.

The initial noninteractive preflight stopped before any benchmark because `sudo` reported `sudo: a password is required`.
The owner then completed the approved A/B/A experiment with terminal authentication.
All phases completed with the original workload and ordinary-user benchmark identity.
Each phase retained erasure maximum violations and a nonzero benchmark exit.
The higher minimum request did not prevent the observed hardware-performance reduction.

The normal guard path restored the original policy within the approved window. A fresh policy observation matches the original values.
This verifies normal host restoration, not recovery after a forced kill, kernel failure or competing policy change.
Keep those limits visible. Preserve the detailed phase results and hardware-counter comparison locally.

This experiment does not establish a production solution or maximum-budget acceptance.
Do not repeat the experiment without new owner permission.

The owner used this command as the ordinary user, without a `sudo` prefix:

```sh
python3 -B target/97-cppc-tx6i0tsm/frequency_experiment.py run
```

The controller requests terminal authentication before the first phase.
It starts only the fixed-path guard through `sudo`. Benchmark processes remain under the ordinary user.
The [diagnostic contract](design.md#approved-temporary-cpu-frequency-experiment) defines the recovery and acceptance limits.
Local preparation evidence stays under `target/97-cppc-tx6i0tsm/frequency-preparation/`.
The completed run and its review stay in a separate frequency-run directory beside that preparation evidence.

## Approved read-only CPU-limit diagnostic

The owner approved the [read-only CPU-limit investigation](design.md#approved-read-only-cpu-limit-investigation).
Prepare `target/97-cppc-tx6i0tsm/msr_observer.py` with an ordinary-user controller, fixed privileged reader and isolated tests.
Record source, executable, policy and render hashes before measurements. Preserve earlier evidence separately.
Keep all system settings unchanged. The completed frequency experiment must not run again.

Test the read-only device boundary, unavailable data, bounded output, cancellation and authentication before a hardware run.
Require private terminal authentication before the control. Never start a benchmark as root.
Run one unchanged control and one observed CPU-12 diagnostic, with separate commands, exits, raw counter readings and policy observations.
Preserve maximum violations even when an older exit condition succeeds.
Report approximate time associations and observer effects. Do not infer a cause from a flag that remains set during normal blocks too.
If authentication fails, stop before hardware reads or workloads and retain the preparation evidence.
The hardware cause, maximum-budget requirement and owner acceptance remain open until separate evidence closes them.

The local diagnostic passes 19 isolated tests with host MSR opens blocked.
They cover the fixed read-only descriptor, device identity, exact read widths, missing data, valid zeros and bounded output.
They also cover authentication failure, root refusal, cancellation, actual pipe closure, partial startup and nonzero reader exits.
The initial noninteractive preflight stopped at authentication with `sudo: a password is required`, before MSR access or either workload.
The owner then completed the approved pair with private terminal authentication.
The following command records that completed run, not a request to repeat it.
The owner started the controller as the ordinary user, without a `sudo` prefix:

```sh
python3 -B target/97-cppc-tx6i0tsm/msr_observer.py run
```

Only the fixed reader uses `sudo`. It performs no register or policy writes and clears no status flags.
Its 55-second deadline cannot guarantee completion if a kernel read stops responding.
The ordinary-user controller preserves partial output and failures. Do not start another pair automatically.
Preparation evidence stays under `target/97-cppc-tx6i0tsm/msr-preparation/`.

The completed pair and independent audit stay under `target/97-cppc-tx6i0tsm/msr-a58a5173ce98469cb53d269c62627336/`.
Both phases completed the original call counts and had no wall-time or CPU-time maximum violations.
The workload ran as the ordinary user. The reader returned all six registers without read errors and ended normally after parent-pipe closure.
Policy observations and the final readback preserve the original policy. Owner input hashes remain unchanged during the audit.
Current limit-status signals occurred in the observation interval, but the pair captured no failing callback.
The hardware cause and historical violations remain open. These passing runs do not close acceptance.
Evidence files retain read durations and polling gaps. They do not directly measure observer impact on CPU 12.
Live failure cleanup, missed short events, owner listening and controller checks remain unverified.
The permission covers only this completed pair. Do not repeat the diagnostic without new owner permission.

## Owner trial feedback and next-editor direction

The owner reported preliminary success with recording and unclear erasure without a pattern editor.
The owner accepted the [event strip and step-selection direction](design.md#owner-trial-feedback-and-deferred-editor-direction).
The strip shows the selected pad's events. Explicit edit mode shows all pad onsets at the selected step and permits targeted removal.
The playback position and edit cursor remain distinct. The 16-cell example is not a fixed grid limit.
Retain the existing straight and triplet divisions, raw timing, velocities and Gate lengths in the next design.
Resolve page size, off-grid events, several events per cell, insertion and meter scope before implementation.
This feedback does not activate 98, change the scope of 97 or close owner acceptance and historical callback failures.
No implementation task or manual check receives a completed mark from this feedback alone.

## Workflow follow-up

- Plan a separate solution for [quiet audio-input recordings](design.md#deferred-audio-input-level-work).
  The owner needs level correction beyond the current +6 dB pad-gain limit. No input measurement or implementation choice exists yet.
- Keep 96 and 97 commits separate on the owner-requested continuation branch.
- The 96 solo-preview question outside recording remains separate from this recording-specific Chop rule.
- Publication, archive and merge require separate owner permission.
- Physical SMC-PAD measurement, factory holds and the full controller reach pass remain outside this change.
- Historical callback maxima above 50 percent remain open even if a later benchmark passes.
