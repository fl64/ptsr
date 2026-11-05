# Tasks

Each implementation task includes its named scenario tests, a successful `task check` and a separate signed-off commit.
Do not combine the preparation move with behavior changes.

## 1. Specification

- [x] 1.1 Record the approved scope, owner decisions, compatibility exceptions, risks, scenario deltas and current status. Check OpenSpec and English text.
- [x] 1.2 Review the defaults in design Open questions with the owner. Record decisions before the affected implementation step.

## 2. Behavior-neutral preparation

The move covers 12 application snapshot methods. Frame assembly, input routing and shared drawing helpers stay in place.

- [x] 2.1 Read the full affected source and tests. Agree the implementation file plan. Move screen-data preparation into `crates/app/src/view.rs` without behavior changes.
- [x] 2.2 Prove unchanged screen identity, title positions, help, notices, save state and keyboard/raw-MIDI action results through existing production routes.

Preparation verification: all 12 method bodies and existing call sites stay byte-identical. Existing tests keep their assertions.
`task check`, `task build` and strict OpenSpec validation pass. Debug tests and Release artifacts use this worktree's sources.
This move adds no behavior or new scenario. The visual scenarios below still need implementation and tests.

## 3. Badges and pad states

- [x] 3.1 Add the shared badge composition and width rules for existing modes. Cover every badge and overlapping placement state.
- [x] 3.2 Add pad corner marks and yellow grid outlines. Preserve hit, selection, bank, hidden-grid and existing swap behavior.
- [ ] 3.3 Retain the fixed PLAY slot and actual-current-project voice checks. Update `docs/ui/STYLE.md` with the approved yellow meaning.

Title verification (3.1): `task check` and `task build` pass with this worktree's Debug and Release sources.
The suites pass: 695 App, 80 input, 106 drawing and 18 benchmark-report tests. Main-spec scenario coverage also passes.
New tests cover keyboard/raw-MIDI mode changes, all seven placement combinations, F12 learning and controlled recording elapsed time.
Drawing tests cover 80×26 and 120×40, compact labels, long Unicode filenames, chop variant identity, fixed playback columns and a scrolled Settings list.
The style guide now defines yellow PadMode titles. Task 3.3 keeps its final pad-state regression check after 3.2.
The later pad-state step below adds corner marks and yellow outlines. Notice history, F1 Messages, value popups and the catalogue remain unimplemented.
Automated verification used no live audio or MIDI devices. Sound processing stays unchanged, so this step adds no sound render.

The owner accepted the title step except Tab under open F1 help on Pads.
Tab still changes the underlying pad parameter page. This exception stays open under task 4.4.
This acceptance does not cover unfinished features or establish separate hardware and listening results.

Pad-state verification (3.2): `task check` and `task build` pass with this worktree's Debug and Release sources.
The suites pass: 700 App, 80 input, 108 drawing and 18 benchmark-report tests. Main-spec scenario coverage also passes.
Five new App tests cover target rules, loading, copy cancellation, whole-file identity, marking, assignment, learning and hidden grids.
Placement and assignment run through keyboard, raw MIDI notes and learned buttons. All seven overlapping placements retain their existing results.
Assignment keeps slice audition. A plan, its numbers and cancellation do not apply changes to pads.
Two new drawing tests cover state colors, bank keys, Unicode names, slice 128, unchanged geometry and hidden grids at both supported sizes.
The style guide records the corner marks and frame priority. Task 3.3 retains its separate final playback regression check.
Automated verification opened no live audio or MIDI devices. This drawing-only step adds no sound render or hardware acceptance.

Owner checks for 3.2: use the keyboard first. Repeat with existing learned controller actions where available.
Use the keyboard for commands without a controller binding.

- On Pads, select a loaded pad. Start copy with Ctrl+C. Check `◎` on an empty target and `!` on an occupied target.
  Hit the occupied target. Check the existing replacement question. Press Esc twice to cancel the question and copy.
- Start swap with Ctrl+X. Hit another occupied target. Check `◎` and the direct swap without another question. Undo with Ctrl+Z.
- Enable marking with M. Play pads and check the yellow frames, audible hits, white flashes and marked fill. Use Esc to cancel.
- On Chop, start with pads visible. Hide pads with Ctrl+O. Start assignment with P. Place two targets and check their slice numbers and existing slice audition.
  Cancel with Esc. Check that the hidden layout returns. No new WAV needs listening.

## 4. Notices and F1 Messages

- [ ] 4.1 Add level, origin, lifetime, bounded pending queue and latest-50 history. Test controlled-time priority, interruption, overflow and owner acknowledgement.
- [ ] 4.2 Classify each existing transient and startup message. Preserve editor prompts, fallback hints, no-key-releases acknowledgement and silent successful autosave.
- [ ] 4.3 Preserve five-second manual save success and persistent save state. Add keyboard/raw-MIDI save refusal, priority, retry and project-refusal regression tests.
- [ ] 4.4 Add F1 Messages and the agreed local page and scroll controls. Test history limits, short windows, origins and no accidental edits or sound commands.

## 5. Values and status

- [ ] 5.1 Add the shared one-second value popup, object labels and numeric range bars. Preserve local highlights and scope-limit counts.
- [ ] 5.2 Carry actual input-source metadata through production routes. Test two devices, direct/deferred events, disconnection, stale events and reconnect.
- [ ] 5.3 Add the shared recording/input/audio/project segments and narrow-width rules. Preserve save state, audio loss, nonzero xruns and no-key-releases warnings.
- [ ] 5.4 Cache unchanged badge and status content. Prove timers, recording elapsed time, level meters and fallback hints still update.

## 6. Questions and screen catalogue

- [ ] 6.1 Show ASK and effective answer controls for each existing question. Test owner key overrides and actual connected bindings without adding `ui.confirm`.
- [ ] 6.2 Protect text overlays from kitty images and restore images after dismissal or expiry. Test graphics on/off commands without real audio.
- [ ] 6.3 Add the checked screen/mode matrix and `task screens`. Render ANSI files at 120×40 and 80×26. Fail tests for missing entries or clipped required content.
- [ ] 6.4 Check every delta scenario against a named test. Run shared scenarios with keyboard and raw MIDI through actual profiles and dispatch.

## 7. Handover and acceptance

- [ ] 7.1 Update user stories, manual checks and applicable existing documentation. Name catalogue files and give separate keyboard and controller procedures.
- [ ] 7.2 Run `task check`, `task build`, `task openspec:check` and `task screens` with verified source provenance. Keep raw diagnostic files local.
- [ ] 7.3 Present the catalogue and manual kitty on/off checks to the owner. Keep hardware and listening results separate from overall acceptance.
- [ ] 7.4 Archive only after owner acceptance and main-spec scenario coverage. Publication and merge need separate authorization.
