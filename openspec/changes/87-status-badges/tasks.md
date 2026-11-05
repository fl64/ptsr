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
- [x] 3.3 Retain the fixed PLAY slot and actual-current-project voice checks. Update `docs/ui/STYLE.md` with the approved yellow meaning.

The owner approved the step 3.3 file plan: extend application input tests and drawing tests, then record verification here.
Production code, sound processing, input routing and the style guide stay unchanged in this step.
The style guide already records the approved yellow meaning from steps 3.1 and 3.2.

Playback review boundaries for 3.3:

- Active modes beside a voice on another bank: closed by combined production-input coverage.
- A mode remains after the last voice ends: closed by callback output and unchanged feedback checks.
- Fixed playback cells beside corners, numbers and mode frames: closed by drawing coverage at both supported sizes and graphics settings.
- Old-project snapshots: closed by an actual old voice snapshot before the next callback and by transition checks with mode feedback.
- Silent requests, previews and effect tails: existing production-path regression tests pass without changes.

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

The owner accepted step 3.2 and requested the next step.
This acceptance does not establish separate controller or listening results and does not cover unfinished features.
Tab under open F1 help remains an open exception under task 4.4.

Owner checks for 3.2: use the keyboard first. Repeat with existing learned controller actions where available.
Use the keyboard for commands without a controller binding.

- On Pads, select a loaded pad. Start copy with Ctrl+C. Check `◎` on an empty target and `!` on an occupied target.
  Hit the occupied target. Check the existing replacement question. Press Esc twice to cancel the question and copy.
- Start swap with Ctrl+X. Hit another occupied target. Check `◎` and the direct swap without another question. Undo with Ctrl+Z.
- Enable marking with M. Play pads and check the yellow frames, audible hits, white flashes and marked fill. Use Esc to cancel.
- On Chop, start with pads visible. Hide pads with Ctrl+O. Start assignment with P. Place two targets and check their slice numbers and existing slice audition.
  Cancel with Esc. Check that the hidden layout returns. No new WAV needs listening.

Playback verification (3.3): `task check`, `task build` and `task openspec:check` pass.
The suites pass: 701 App, 80 input, 109 drawing and 18 benchmark-report tests. Main-spec scenario coverage also passes.
The new App test covers 72 combinations of keyboard, MIDI notes, learned buttons, modes, hidden grids and voice completion or project transitions.
It observes actual offline callback output and voice snapshots. Direct MIDI stays on the real callback queue, without capture or replay.
A queued hit does not light PLAY. A voice on another bank keeps it active, but a pending mode alone does not.
The transition test retains a real old voice snapshot before the next callback. PLAY still excludes that snapshot.
The new drawing matrix covers 80×26 and 120×40, both graphics settings, edit, hidden grids, marks, placement, learning and assignment numbers.
PLAY and LISTEN retain their ten-cell slots, styles and following columns.
Changing only playback state keeps other fixture cells and image rectangles unchanged.
Assignment audition retains its existing idle LISTEN slot. The earlier preview and effect-tail tests stay unchanged and pass.
Debug tests, the Release build and the executed callback benchmark use this worktree's sources.
Production code and the approved style guide stay byte-identical. No new sound feature, WAV or live device check is part of this step.
Tab under open F1 help still needs task 4.4. Hardware, listening and unfinished-feature acceptance remain separate.

## 4. Notices and F1 Messages

The owner approved the five-file step 4.1 plan: a new notice model, its test target, this task list, design and screen-help delta.
Implement the model first. The current message line and input routing stay unchanged until the integration steps below.
The model tests load the future application source directly, without a second implementation or live devices.
Cover exact deadlines, hidden visible time, interruption, production order, overflow, independent history and explicit Error acknowledgement.
Run `task check`, `task build`, strict OpenSpec validation and English checks before the local signed-off commit.
No owner action, listening render, publication or F1 navigation change is part of this step.

- [x] 4.1 Add level, origin, lifetime, bounded pending queue and latest-50 history. Test controlled-time priority, interruption, overflow and owner acknowledgement.
- [x] 4.2 Classify each existing transient and startup message. Preserve editor prompts, fallback hints, no-key-releases acknowledgement and silent successful autosave.

The owner approved the 17-file plan for 4.2: application main, views, dispatch, MIDI, saving and notices, plus their input/save/model tests and drawing.
The remaining files are this task list, design, screen-help delta, OpenSpec context, project/plan status and the style guide.
The owner then approved `crates/input/src/midi.rs` as the eighteenth file. Effective resolution supplies activity without changing the action result.
Record levels at all 79 existing slot producers and keep ongoing state separate. Preserve startup acknowledgement exceptions before connecting the model.
Extend production keyboard/raw-MIDI, controlled-time and drawing tests. Do not substitute a last-device guess or consume acknowledged input.
Keep F1 navigation, value popups, the shared input indicator, sound, factory bindings and saved formats outside this step.

- [ ] 4.3 Preserve five-second manual save success and persistent save state. Add keyboard/raw-MIDI save refusal, priority, retry and project-refusal regression tests.
- [ ] 4.4 Add F1 Messages and the agreed local page and scroll controls. Test history limits, short windows, origins and no accidental edits or sound commands.

Model verification (4.1): `task check`, `task build` and `task openspec:check` pass.
Seventeen new notice tests pass in both Debug and Release. They use controlled time without sleeps or live devices.
The existing suites pass: 701 App, 80 input, 109 drawing and 18 benchmark-report tests. Main-spec scenario coverage also passes.
The model covers all 16 level pairs, exact deadlines, nested interruptions, equal-level replacement and original production order.
It pauses hidden notice time and gives the next entry its remaining lifetime after a delayed update.
Overflow tests retain the active Error, bound pending entries at 50 and check the separate latest-50 history in exact order.
Repeated reads, resumption, expiry and explicit acknowledgement add no history entries. Repeated requests still get separate entries.
The model retains the supplied keyboard, device or system origin. A new instance has no previous history.
Debug, Release model tests, the Release sampler and the executed callback benchmark use this worktree's source paths.
The sampler build reuses unchanged Release code. The new model is not part of that executable yet.
Application drawing, input dispatch, saving, engine, bindings, dependencies and existing tests stay unchanged.
Production acknowledgement, message levels and source routing still need 4.2 and 4.3. F1 Messages and its controls still need 4.4.
The owner has no manual check for 4.1. No new WAV, hardware result, publication or archive is part of this step.
Tab under open F1 help remains an open exception for correction in 4.4.

Migration verification (4.2): `task check`, `task build` and `task openspec:check` pass.
The suites pass: 708 App, 81 input, 110 drawing, 21 notice-model and 18 benchmark-report tests. Main-spec scenario coverage also passes.
All 21 model tests pass in Release. Debug tests, the Release sampler and the executed callback benchmark use this worktree's sources.
The producer inventory contains 79 former one-slot writes. Their outcomes now supply levels directly, without text matching.
Startup and keymap tests retain their distinct acknowledgement rules and one history record per production.
Two real profile routes retain the producing port through alternating commands, parameter changes and deferred pad placement.
Only the raw-message envelope acknowledges once. Derived actions retain normal routing, velocity and release ownership.
Shown-token tests retain unseen and identical replacement Errors. Real filter input hides notices without blocking typing or losing the Error.
Input tests distinguish learned button releases from continuous movement to zero. Repeated knob values do not acknowledge Errors.
Loading tests preserve unrelated Errors and reject old-project replies. One frame clock keeps exact expiry, text, level and keymap visibility consistent.
Drawing tests check all four cues at 80×26 and 120×40 without changing other rows.
Existing save tests retain five-second manual success, silent successful autosave, refused bytes, read protection and retry behavior.
Task 4.3 still needs its dedicated save-priority and persistent-status regression extension. F1 history and the Tab exception remain in 4.4.
Automated verification opened no live audio or MIDI devices. Sound processing, factory bindings, dependencies and saved formats stay unchanged.
No new WAV, hardware result, owner acceptance, publication or archive is part of this step.

Owner checks for 4.2, not yet performed:

- On Pads, select an empty pad and press Ctrl+C. Check the `!` warning. Wait six seconds without another action and check the normal hint.
- Press Ctrl+S in a writable disposable session. Check cyan `✓ Session saved` for five seconds.
- Repeat with existing learned copy and save controls, where available. Button releases must not dismiss an Error.
  Physical-controller and listening results remain separate. No new WAV needs listening.

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
