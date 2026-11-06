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

- [x] 4.3 Preserve five-second manual save success and persistent save state. Add keyboard/raw-MIDI save refusal, priority, retry and project-refusal regression tests.
- [x] 4.4 Add F1 Messages and the agreed local page and scroll controls. Test history limits, short windows, origins and no accidental edits or sound commands.
- [x] 4.5 Keep Help and Messages in one stable window. Test shared frame, labels and hints across content changes and terminal resizing.

The owner approved thirteen files for step 4.4:
application main, dispatch, views, notices, input tests and a new `crates/app/src/help.rs`, plus the drawing crate.
The six artifact files are design, this task list, the screen-help delta, project status, plan status and OpenSpec context.
Record the scrolling defaults before code. First add a production-keyboard regression for Tab behind open help.
Preserve higher-priority input, learned help controls and existing keyboard/raw-MIDI pad routes.
Test empty and latest-50 history, origins, wrapped Unicode, new results, resize, repeat and close/reopen behavior.
Compare saved work, project identity, undo and sound commands across local navigation.
Run `task check`, `task build`, `task openspec:check` and English checks before the implementation commit.
Do not change sound, bindings, dependencies, saved formats, deferred copy names or the later kitty-overlay scope.

Source verification before 4.4 found reused executables from the separate 87a worktree in the shared target directory.
That first run does not establish this branch's results. The acceptance record commit contains no code change.
After package cleanup, the current sources pass `task check`: 711 App, 81 input, 110 drawing and 21 notice-model tests.
A separate Release cleanup and `task bench` compile and execute the current engine sources.
These fresh results support the acceptance record without a history rewrite.
Earlier callback maximum-budget failures remain unresolved. Raw build and benchmark evidence stays local.

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

Save feedback verification (4.3): `task check`, `task build` and `task openspec:check` pass.
The owner approved three files: application input tests, save tests and this task list.
The suites pass: 711 App, 81 input, 110 drawing, 21 notice-model and 18 benchmark-report tests. Main-spec scenario coverage also passes.
Three new tests cover six interrupted-save runs, eighteen refusal/retry runs and six save-state runs.
Keyboard, raw MIDI notes and learned CC buttons use the production profiles and dispatch. Each produced result retains its actual origin.
The save notice uses five visible seconds, including interruption and the exact resumed deadline.
Repeated refusals retain separate history entries without resetting the queue. Background Info and Done results cannot replace the active Error.
Acknowledgement and later notice expiry retain pending work, read protection and the persistent write-failure status.
Restored write access does not resume automatic or exit saves. A successful manual retry permits later silent autosave.
Valid file bytes alone do not permit saving.
Extended transition tests retain source work, target bytes, picker state and typed names through repeated refusals and background notices.
Picker confirmation keeps its keyboard path, so transition refusal results correctly record Keyboard after either picker-entry source.
Debug tests and Release artifacts use this worktree's sources. The Release build reuses unchanged application code.
Production code, sound processing, bindings, dependencies and saved formats stay unchanged. This step adds no WAV or live-device check.
The callback benchmark command passes, but its maximum-budget acceptance remains open. Raw results stay local.
F1 Messages and the Tab exception remain in 4.4. Hardware, kitty and listening checks remain unverified.

The owner accepted step 4.3 at commit `1682a20`.
This acceptance does not establish separate hardware, kitty or listening results and does not cover unfinished features or the callback maximum-budget gate.

F1 verification (4.4):
- F1 opens Help. Tab/Shift+Tab selects Help or Messages. Messages shows the latest 50 process-local results, newest first, with levels and actual origins.
- Up/Down scrolls one wrapped row. PageUp/PageDown scrolls one visible page. Resizing clamps the offset. New results keep the row offset.
- Tab no longer changes the parameter or page behind help. F1/Esc closes help without leaving the existing edit layer.
- Existing text, picker, question and learn input retains priority. Keyboard, raw MIDI note and learned-button pad presses and releases keep their routes.
- No saved format, new binding, dependency, sound code or deferred copy-name behavior changed. This drawing/input step needs no new audio render.

Step 4.4 evidence:
- The new navigation regression first failed: Tab changed Volume to Attack under open F1 help. It passes with local help navigation.
- Seven new App tests cover all five screens, both terminal sizes, 50/51 results, source labels, arrivals, resize, repeat/release input and reopen reset.
- The tests also cover existing nonempty pad/undo state, pending work, project identity, input priority and an Error that arrives under help.
- Two new drawing tests cover wrapping, levels, empty history, full scroll reach and tiny terminals.
- `task check`: passes, including 718 App tests, 81 input tests, 112 drawing tests and 21 notice-model tests.
- `task build`: passes. The Release compiler used this worktree. No live audio or MIDI device opened.
- `task openspec:check`: passes, 16 changes and specifications, 0 failures.
- English review: STE-flavored for descriptions and Strict for the manual procedure. Revised text has no lint violations.
- Full status-document lint retains one existing lexical finding: the plans heading uses `verify`, with `check` elsewhere. That unrelated heading stays unchanged.
- Local commands and logs are in `target/87-f1-*`. Callback diagnostics stay local.
- A successful callback run does not resolve earlier maximum-budget failures. That acceptance gate remains open.

Step 4.4 owner procedure (pending):
Switch off wave pictures in F12 (`Wave`) for this procedure. Kitty image composition remains in task 6.2.

1. Open a disposable scratch session or project. Save with Ctrl+S. Note the selected pad and parameter.
2. Press F1, then Tab. Check `Session saved`, its Done level and Keyboard origin in Messages.
3. Switch pages with Tab and Shift+Tab. Scroll Messages with Up/Down and PageUp/PageDown. Resize to 80x26 and 120x40.
4. Close with Esc. Check that the selected parameter, bank and pending work did not change through help navigation.
5. In a terminal with key releases, hold a gate pad. Open help. Release the pad. Close help. Press and release the pad again.
6. Repeat step 5 with the controller. Listen for both pad stops. Physical-controller and listening results remain pending. No new WAV accompanies step 4.4.

Step 4.4 closes the automated F1/Tab defect, not the owner's separate manual acceptance.

Stable F1 window correction (4.5):
The owner reported a changing frame size when switching pages and approved one common window.
The approved four files are the screen-help delta, design, this task list and `crates/tui/src/lib.rs`, including its drawing tests.

Both pages use the previous Messages body. Page labels and shared hints keep fixed positions.
Short help can leave unused space. Only terminal resizing changes the common frame dimensions.
Input, sound, dependencies, saved formats, copy naming and kitty image handling stay unchanged.

Step 4.5 evidence:
- Three new drawing regressions failed against the previous frame layout and pass after correction.
- Geometry checks cover all five screens, 0/1/16 help rows, 0/1/50 messages and repeated page switches at both catalogue sizes.
- Other checks cover fixed labels and hints, active styles, Messages-only scroll hints, resize in both directions and the last message.
- Existing Unicode wrapping, tiny-terminal and keyboard/raw-MIDI regression tests still pass.
- `task check`: exits 0, including 718 App tests, 81 input tests, 115 drawing tests and 21 notice-model tests.
- `task build`: passes. Release compilation used the current worktree's drawing and application sources.
- `task openspec:check`: passes, 16 changes and specifications, 0 failures.
- English review uses STE-flavored descriptions and a Strict manual procedure. Existing advisory findings stay unchanged.
- The callback benchmark command exits 0. Maximum-budget acceptance remains open. Measurements, CSV files and logs stay local under `target/`.
- No live audio or MIDI device opened. Sound processing stays unchanged. This step needs no new WAV.

Step 4.5 owner procedure:
1. With wave pictures off, open F1 at 80x26.
2. Alternate Tab and Shift+Tab. Check that the frame, page labels and shared hints do not move.
3. Repeat at 120x40. Resize with each page open. Check that both pages follow the terminal size.
4. Open Messages. Check Up/Down and PageUp/PageDown. Close with Esc. Check that the selected parameter and bank stay unchanged.
5. Repeat on Pads, Chop, Files, FX and Settings.

The owner accepted the stable Help/Messages window at commit `1e7a9b2`.
This acceptance covers the shared frame, not separate physical-controller, kitty or listening results.
It does not cover unfinished features or the callback maximum-budget gate.

## 5. Values and status

The owner approved the twelve-file plan for step 5.1 before implementation.
Application files: main, dispatch, views, input tests and new `crates/app/src/values.rs`.
Drawing uses `crates/tui/src/lib.rs`. Artifacts are design, this task list, the screen-layout delta, project/plan status and OpenSpec context.
Observe existing setters through production keyboard and raw-MIDI routes. Add no parallel setter or new sound command.
Test applied values, exact elapsed-time deadlines, equal repeats, pickup, refusal, partial scope limits and stable object identity.
Test editor precedence, both supported sizes, Unicode and restoration without another action.
Run `task check`, `task build`, `task openspec:check` and English checks before the signed-off commit.
Shared input metadata, status caching, kitty commands, deferred copy names and callback correction remain outside this step.

- [x] 5.1 Add the shared one-second value popup, object labels and numeric range bars. Preserve local highlights and scope-limit counts.
- [x] 5.2 Carry actual input-source metadata through production routes. Test two devices, direct/deferred events, disconnection, stale events and reconnect.
- [x] 5.3 Add the shared recording/input/audio/project segments and narrow-width rules. Preserve save state, audio loss, nonzero xruns and no-key-releases warnings.
- [x] 5.4 Cache unchanged badge and status content. Prove timers, recording elapsed time, level meters and fallback hints still update.

Step 5.1 verification:
- Applied values use one bounded panel below the title. Numeric values have range bars. Categorical choices retain their words.
- Twelve new production-input tests cover all five screens, pad parameters, shared FX, Chop, Settings, pickup, refusal, scopes and editor priority.
- A separate formatting test checks buffer choices and actual routed output labels without claiming a live sound-server result.
- Exact-deadline tests retain the original timestamp on equal values or limits. Later actual changes restart the deadline.
- The UI frame releases expired sample snapshots. Expiry restores the underlying text without another input.
- Drawing tests cover 80x26, 120x40, Unicode, tiny terminals, range endpoints and unchanged cells outside the popup.
- `task check`: exits 0, including 731 App, 81 input, 117 drawing and 21 notice-model tests.
- `task build`: exits 0. Current-worktree application sources compile in Release.
- `task openspec:check`: passes, 16 items and 0 failures. Long requirement text now has separate identity, deadline and priority requirements.
- English review uses STE-flavored descriptions and a Strict owner procedure. Revised text has no new lint findings.
- Existing full-file advice stays unchanged. The plans retain their earlier `verify`/`check` lexical finding.
- Earlier callback maximum-budget failures remain unresolved. A later passing measurement does not close that gate.
- Logs and diagnostics stay local under `target/87-value-*`. No live audio or MIDI device opened.
- Sound processing, input bindings, saved formats, dependencies and deferred copy names stay unchanged. This step adds no WAV.
- Kitty image protection remains in 6.2. Hardware, listening and owner acceptance of 5.1 remain open.

Step 5.1 owner procedure (pending):
1. Select Braille for `Wave` in F12 Audio. Restart the program if wave pictures remain active.
2. Open a disposable session or project. On Pads, select a loaded pad. Press Right once.
3. Check the upper-right panel: pad bank/key, volume, applied dB and range bar. Wait without input.
4. Check that the panel ends after one second. The existing bottom notice can remain longer.
5. Open Chop with F3. Move the cursor with J/L. Change slice bounds with Ctrl+J/L or Alt+J/L.
6. Open FX with F4. Change a value with J/L. Check the shared FX slot label.
7. On Files and Settings, press `]`. Check the Master volume panel.
8. Repeat applicable value changes with existing controller knobs. With pickup active, check that waiting does not show an unapplied value.
9. Check the existing pad sound while you change values. No new WAV accompanies this drawing step.

Step 5.2 approved scope:
- Application files: `main.rs`, `dispatch.rs`, `midi.rs`, `input_tests.rs` and new `input_sources.rs` in `crates/app/src/`.
- Artifacts: design, this task list, the screen-layout delta, `docs/PROJECT.md`, `docs/plans/README.md` and `openspec/config.yaml`.
- Add original receipt times and fresh connection identities without changing commands, project stamps, acknowledgement or release ownership.
- Test production keyboard and recorded MIDI, two devices, delayed notifications, first input before Connected, stale generations and missing metadata.
- Test exact deadlines without sleeps. Keep source state across project replacement, not application restart.
- Run `task check`, `task build`, strict OpenSpec and English checks. Make one local signed-off commit.
- Visible status segments, compact labels, caching, kitty commands, physical devices and callback correction remain outside this step.

Step 5.2 verification:
- Ten new production-route tests and three source-model tests cover identities, activity and lifetimes. Source checks use all five screens.
- Reordered envelopes cover direct/deferred work, two devices, original velocities, release ownership, disconnect, reconnect and first input before Connected.
- Exact deadlines use recorded receipt times without sleeps. Releases, repeats, equal values and old deliveries do not renew them.
- Two knob regressions failed before the changes. The model now compares values within their connection, channel, control and receipt order.
- Missing metadata stays unknown without blocking existing actions. Project replacement keeps source state. A new application starts without it.
- Old-project payloads remain discarded without acknowledging the shown Error. Source metadata still retains the actual device and receipt time.
- `task check`: exits 0, including 744 App, 81 input, 117 drawing and 21 notice-model tests.
- `task build`: exits 0. Current-worktree application sources compile in Release.
- `task openspec:check`: passes, 16 items and 0 failures. Identity, lifetime and continuous values have separate requirements.
- English review uses STE-flavored descriptions. Revised text has no new lint findings. Earlier full-file advice stays unchanged.
- Final diff contains only the eleven approved files. Logs remain local under `target/87-source-*`.
- No live audio or MIDI device opened. Sound processing, saved formats, dependencies, bindings and deferred copy names stay unchanged. No new WAV.
- The visible status row remains in 5.3. Keyboard and controller visual acceptance follows that step, not this metadata-only step.
- Earlier callback maximum-budget failures remain unresolved. A later passing measurement does not close that gate.

Step 5.3 scope:
- Keep the shared status visible during learning. Move learning context, detail and next steps into the fixed three-row help band.
- Read actual source snapshots from 5.2. Add distinguishable full/compact labels and elapsed recording time.
- Preserve safety indicators before optional project, audio and input details. Test Unicode, control characters, long labels and extreme counters.
- Application changes stay in main, views, input sources and input tests. Drawing changes stay in the existing drawing crate.
- Update the approved artifacts and the additional MIDI-learn delta before code. Keep sound, routing, bindings, formats and dependencies unchanged.
- Run project checks, Release, strict OpenSpec and English checks before the local signed-off commit.

Step 5.3 verification:
- The final `task check` exits 0: 747 App, 81 input, 120 drawing and 21 notice-model tests pass.
- The current-source Release build passes in 26.24 s. Strict OpenSpec passes all 16 items.
- Tests combine failed saves, recording, no audio, nonzero xruns and the full no-key-releases phrase at both supported sizes, including extreme counters.
- Real keyboard and recorded raw MIDI exercise source labels on all five application screens. Drawing tests cover the shared row on each screen.
- Source labels follow connection identities when their compact numbers change. Highlights expire at their original deadline without another action.
- Learning keeps its status and three-row band. Pending controls remain silent and unsaved until confirmation. Cancellation preserves assignments.
- Effective learning hints follow owner key overrides. Files has no parameter-selection hint. A clear request replaces stale refusal feedback.
- New descriptions use STE-flavored review. Revised text has no lint findings. The MIDI-learn delta retains one advisory in an unchanged scenario title.
- An earlier run in this step reports wall max 96.25%, CPU max 46.40% and one wall sample above 50%.
- The final run reports wall max 38.73% and CPU max 38.40%. It does not close the earlier callback maximum-budget gate.
- No live audio or MIDI device opened. Sound, bindings, saved formats, dependencies and deferred 87a stay unchanged. No new WAV.
- Owner visual, physical-controller and listening acceptance remains unverified. The owner authorized publication, but not merge or archive.

Step 5.4 verification:
- The final `task check` exits 0: 750 App, 81 input, 122 drawing and 21 notice-model tests pass.
- The current-source Release build passes in 30.94 s. Strict OpenSpec passes all 16 items.
- Badge formatter counts do not increase on identical frames. Real keyboard/raw-MIDI screen, mode, page and recording inputs refresh the necessary labels.
- Input labels share one immutable array until connection identity or activity changes. Reconnect and project-replacement tests retain correct source lifetime.
- Status-row tests cover every cached field, both sizes and CPU rounding. Notice, meter, playback and hint updates do not reconstruct static status text.
- Controlled-time tests expire value popups, input highlights and notices without another action. Recording seconds advance through the same production cache.
- The initial screen test incorrectly used F3 from Files as a pad-screen shortcut. Its corrected independent fixtures pass without a production-route change.
- No live audio or MIDI device opened. Sound, input ordering, bindings, saved formats, dependencies and deferred 87a stay unchanged. No new WAV.
- This reuses text. It does not remove all UI allocations or close the earlier callback maximum-budget failures.

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
