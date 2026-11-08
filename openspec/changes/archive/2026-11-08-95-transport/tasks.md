# Tasks

## 1. Audio-thread transport

- [x] 1.1 Add clock commands and exact accented metronome without using pad voices. Cover default state, play/stop, ordered toggles, tempo changes, mute phase, empty blocks and 32-voice preservation with automated renders.
- [x] 1.2 Publish bounded coherent transport snapshots. Cover concurrent readers, project reset, stale commands and saturated queues with real-time tests. Extend allocation/deallocation checks through transport commands and rendering.
- [x] 1.3 Generate `renders/95-transport.wav` with timing, finite-value, level, silence and accent assertions. Measure active transport with full polyphony while preserving existing benchmark limits.

## 2. Persistent preferences and input

- [x] 2.1 Add format 4 transport preferences with formats 1–3 migration. Test round trips, invalid-value refusal, scratch persistence and preservation of pad/chop/group/effect state.
- [x] 2.2 Add keyboard actions and learned MIDI controls through production routing. Test both keyboard layouts/protocols, repeats/releases, owner overrides, old action names and independent knob takeover.
- [x] 2.3 Restore target preferences stopped through the existing project transition. Test full queues, stale epochs, source-save refusal, target-read refusal and selecting the current project with a running clock.

## 3. Visible controls and integration

- [x] 3.1 Add accepted-state transport feedback, tempo value popup and effective-key hints. Test safety priority, screen changes, learn silence and both supported terminal sizes. Keep owner procedures current in the proposal.
- [x] 3.2 Run `task check`, `task build`, `task screens` and `task openspec:check`. Review all scenario coverage and boundary statuses, local links, changed English text and the complete diff. Record exact results and unverified owner procedures without claiming listening or hardware acceptance.

## Workflow follow-up

- Keep this change separate from repository-only roadmap edits and later pattern changes.
- Commit verified logical steps only with the required permission and green `task check`. Do not add placeholder tests to permit a draft-only commit.
- Physical-controller and listening verification remain owner procedures. Do not access live audio or MIDI devices for these tests.
- Archive only after owner acceptance. No archive, push, PR publication or merge is implied by passing automated tests.

## Implementation verification

Workspace tests and required p99 exit checks pass. An earlier recorded run missed the 50% maximum target. The cause remains unknown.
The owner authorized local commits. These completed tasks describe implementation and automated verification, not individual owner procedures.
These implementation results precede archive. The closure section below records the later owner authorization and archive.

- `task check`: exit 0, 1156 workspace tests, including 802 App tests. All 41 delta scenarios reference executable tests.
- `task build`: exit 0. Release binary: `target/release/ptsr`.
- `task screens`: 4 passed, 0 failed. The offline catalogue remains local under `renders/screens/`.
- `task openspec:check`: `Totals: 16 passed, 0 failed (16 items)`.
- Changed English text, local links, owner shell syntax and `git diff --check` pass. Logs remain local under `target/95-transport/`.

The benchmark includes the real click sample with transport and metronome at 172 BPM, plus the existing 32-voice/effects workload.
Existing limits remain unchanged. In the earlier `check.log` run, steady wall p99 is 13.78% of the budget.
The wall maximum is 93.29%, with 3 blocks above 50%. The thread CPU maximum is 71.39%, with 2 blocks above 50%.
These maxima miss the target even though `task check` exits 0. Do not attribute the peaks to transport without a control run.
The later `pre-commit-check.log` run exits 0 and also meets the maximum target. It does not explain or close the earlier failures.
Performance fixes remain deferred for a separate owner decision. Historical maximum-budget and slow-core investigations remain open.

`renders/95-transport.wav` is local. Its automated checks cover timing, accents, silence, finite samples and level.
The [owner procedure](../../../../docs/LIVE-CHECKS.md#95-transport-and-metronome) separates keyboard, physical-controller and listening checks.
The agent did not open live audio or MIDI. Individual owner listening, terminal delivery and physical-controller results remain unspecified.

## Owner feedback and deferred work

The owner gave positive general feedback after the manual checklist. Do not infer individual controller or WAV listening results from that reply.
The owner deferred separate metronome waveform visuals and diagnosis of a click at Chop preview start. See the non-goals in `design.md`.
Continue transport work without those changes. That deferral does not restore canceled change 109 or authorize publication or archive.

## Owner acceptance and archive (2026-11-08)

The owner approved closure of 95, including specification sync, archive and automated checks.
This is overall acceptance, not individual keyboard, terminal, physical-controller or listening results. Their unreported procedures remain unverified.
The archive preserves the deferred Chop-click diagnosis and separate metronome visuals. Earlier callback maximum-budget failures remain open.

All 15 added requirements and 3 modified requirements match the five main specifications.
All 41 delta scenarios remain present. The main specifications contain 839 scenarios, up from 804.
Unrelated requirements remain unchanged. The three scenario-audit tests pass before archive.
After sync verification, `openspec archive 95-transport --yes --skip-specs` moved the change without changing the main specifications.
Archive: `openspec/changes/archive/2026-11-08-95-transport/`. No change is active.
The owner later authorized push, PR publication and squash merge after required checks pass. Keep the branch and recorded limits.
The separate roadmap edits stay outside this change. Publication does not complete the unverified hardware or listening checks.

### Archive verification

- `task check`: exit 0, 1156 workspace tests passed. The main-spec audit covers 839 scenario names with four unchanged hardware exceptions.
- `task build`: exit 0. Release binary: `target/release/ptsr`.
- `task screens`: `4 passed; 0 failed`. The catalogue contains 306 local ANSI files.
- `task openspec:check`: `Totals: 16 passed, 0 failed (16 items)`.
- Specification comparison, 41-scenario test-declaration audit, 16 local links, owner shell syntax and `git diff --check` pass.

The archive benchmark again exceeded the maximum target despite its successful p99 exit checks:

```text
paired timing: steady playback
  wall: mean 9.69 %, p99 30.53 %, max 54.60 %, samples 20000, >50 % 2
  thread CPU: mean 9.70 %, p99 30.55 %, max 54.65 %, samples 20000, >50 % 2
  maximum target (50 %): wall EXCEEDED; thread CPU EXCEEDED
```

Project-switch wall peak p99 was 31.35%, with maxima of 31.36% wall and 31.40% thread CPU.
No project-switch block exceeded 50%. The steady maximum failures remain unexplained and open. Limits stay unchanged.
Local evidence: `target/95-transport/archive-{check,build,screens,openspec}.log` and `target/95-transport/archive-docs-final.log`.

STE-flavored review and lint pass without hard findings. The context retains one passive-voice suggestion because the actor is irrelevant.
This does not establish full ASD dictionary compliance. The owner shell passed syntax checking only, without execution.
The agent did not open live audio or MIDI. General acceptance does not establish individual terminal, controller or listening results.

### Publication verification

After the authorization update, `task check` exited 0 with 1156 passing workspace tests.
`task build` passed. `task screens` reported `4 passed; 0 failed` and generated 306 ANSI files.
`task openspec:check` reported `Totals: 16 passed, 0 failed (16 items)`.

Steady wall p99 was 31.83%, with maxima of 47.79% wall and 47.21% thread CPU. No steady block exceeded 50%.
Project-switch wall peak p99 was 17.43%, with maxima of 66.21% wall and 66.06% thread CPU.
One project-switch block exceeded 50% for each clock. This again misses the maximum target, despite passing p99 exit checks.
The cause remains unknown. Limits, deferred fixes and unverified owner procedures stay unchanged.
Evidence: `target/95-transport/publish-{check,build,screens,openspec}.log`.
