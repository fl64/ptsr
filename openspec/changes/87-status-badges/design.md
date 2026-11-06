# Visible modes and messages

## Context

The owner approved the full visual scope on 2026-11-05, then approved this specification step and local signed-off commits.
The base is merged save safety and diagnostic work at `a4789f4`.
The owner then approved the navigation and message defaults and the first implementation file plan.
Keep specification decisions and the behavior-neutral preparation in separate commits. Publication and archive need separate authorization.

Evidence at the base commit:

- `crates/app/src/dispatch.rs:832` builds the shared input focus, including text, picker, question, learning and pending placement states.
- `crates/app/src/dispatch.rs:911` preserves different keyboard and MIDI priority when pending placement modes overlap.
- `crates/app/src/main.rs:4128` selects recording, audio failure, a five-second flash and loading before other notices.
- `crates/app/src/main.rs:4178` preserves text-input and keymap-warning priority when preparing a frame.
- `crates/tui/src/lib.rs:2617` puts persistent save state before the rest of the status line.
- `openspec/specs/screen-help/spec.md:164` requires five-second manual save feedback.
- `openspec/specs/pad-playback/spec.md:988` requires the existing fixed-width PLAY indicator and its actual-voice behavior.

The planning documents describe a target. Existing specifications still govern compatibility where the plan omits a case.

## Goals / Non-Goals

**Goals:**

- Identify current modes without making the owner read a title sentence.
- Keep important errors visible and make recent messages available in F1.
- Put changed values, input identity and recording status in predictable places.
- Give existing actions equivalent feedback through keyboard and raw MIDI.
- Provide offline screen evidence without opening audio or MIDI devices.

**Non-Goals:**

- New sound processing, scheduling settings or callback-budget correction.
- New factory bindings, saved formats, dependencies or persistent message logs.
- Generic controller confirmation, holds, relative encoders or a controller profile.
- Controller action echo, device-specific help groups or layer indicators from 92.
- Transport, patterns, blinking future states or command panels.

## Decisions

### 1. Extract preparation before changing behavior

Move screen-data preparation into `crates/app/src/view.rs` in its own commit.
Keep method bodies, current messages, input order, audio requests and effective keys unchanged during that move.
Run existing input, screen and save-state tests before adding new behavior.
Do not combine an input correction with this move.

A separate model for each screen would leave shared priority rules duplicated.
A single large rewrite would make behavior changes harder to isolate.

### 2. Compose badges from current state

Use shared input focus for existing mode flags, plus actual recording and Chop state.
Retain the current screen badge and the fixed ten-cell F2 PLAY and F3 LISTEN slots.
These playback slots are not optional mode badges. Their existing blue active styles stay.
Optional badges have text and semantic kinds: yellow PadMode, cyan On, magenta Attention.
The first optional badge names active pad modes. Attention precedes ordinary On badges after it.
Prefer ASK before REC, then EDIT, LOOP and AS PAD. Use deterministic ordering for ties.
Retain recording in the status line even when a title badge does not fit.

| State | Normal badge | Compact form |
| --- | --- | --- |
| Marking enabled | `MARK · hit pads · N marked` | `MARK N` |
| Marks retained | `N MARKED · Esc drops` | `N MARKED` |
| File pending | `IN HAND · hit a pad` | `IN HAND` |
| Copy pending | `COPY · hit a pad` | `COPY` |
| Swap pending | `SWAP · hit a pad` | `SWAP` |
| Chop assignment | `ASSIGN · N/T placed` | `ASSIGN N/T` |
| Pad or slice edit | `EDIT · <page>` | `EDIT <page>` |
| Screen or MIDI learning | `LEARN` | `LEARN` |
| Existing question | `ASK` | `ASK` |
| Microphone or output recording | `● REC <elapsed>` | `REC` |
| Chop loop or as-pad playback | `LOOP`, `AS PAD` | unchanged |

Do not change input priority to make badges easier to compute.
When pending placement modes overlap, one compound PadMode badge names each active placement mode.
The pad target marks do not promise which source-specific route will take priority.
Text entry and questions keep their current input precedence.
At narrow widths, remove lower-priority optional badges before shortening the remaining labels.
Keep the screen identity and an active pad-mode label visible at supported sizes.

The title step adds `BadgeKind` and `Badge` snapshots in the drawing crate.
Each badge carries its semantic kind, normal text and compact text. `Status` carries the shared badge list from the application.
The application reads existing focus flags without changing them. It combines pending file, copy and swap labels in one badge.
The renderer selects at most three optional badges and reserves space for screen-specific title details.
Remove the separate `Pads.mark_plate` field after its callers use the shared list. Keep its mark counts and yellow style.
Draw badges inside each title, not over a completed screen. Keep title rectangles and playback columns unchanged.
Keep the Settings title outside its MIDI scroll area. A scrolled list then shows two fewer rows, but its selection stays visible.
Pad corners, notice history, F1 page controls and value overlays remain separate steps.

### 3. Add marks without changing pad actions

The owner approved yellow for modes where pads do more than ordinary playback.
Marking still sounds. Learning and questions retain their existing routing.
A yellow grid outline does not replace bank membership, white hit flashes or the selected-pad indication.
When the grid is hidden, keep the mode badge and existing pad line. Assignment retains its current forced-grid rule.

Use `◎` for available file/copy targets and `!` for occupied targets that already require confirmation.
Use `◎` for swap targets without implying a new question: occupied swaps remain direct.
Show planned slice numbers during assignment. Existing mark dots retain their positions.
Never use color as the only distinction between copy, swap, file placement and learning.

The owner approved the pad-feedback file plan before this step.
Carry an immutable grid snapshot beside title badges: a mode-outline flag and one optional corner mark per visible pad.
Draw corner marks at the top right of the existing cell frame. Keep the key, name and bank marks readable.
Tint ordinary cell frames yellow during the mode. Keep existing hit, selected, marked and in-hand styles above that tint.
Keep bank colors on idle key labels even when the mode changes their frame color. Retained marks alone do not enable the mode outline.
Do not add another grid or reserve more rows. The drawing snapshot does not change input focus or saved state.

Read target availability from the existing placement rules, not from a pad name.
A copy source cancels copying, a loading copy target waits, and an identical whole-file placement changes nothing.
Do not advertise these cases as replacement questions. Loading swap endpoints are not available targets.
When pending operations overlap, show `!` if any applicable operation requires confirmation, otherwise `◎` for an available target.
This mark does not select between the existing keyboard and MIDI priorities.
On Chop, assignment numbers follow the actual plan and bank. Listen mode does not advertise pending file, copy or swap targets.
Screen learning retains its existing control labels instead of advertising placement targets.

### 4. Centralize transient notices

Use Done, Info, Warn and Error levels with Keyboard, Device or System origins.
Route existing one-slot messages and startup notices through this model.
Keep persistent save state, recording state, loading progress and editor prompts separate from transient lifetime rules.
A frame does not generate another history entry for the same state.

Done and Info last three seconds of visible time. Warn lasts six seconds.
Manual save success keeps five seconds. Error waits for the next new owner action.
General startup notices retain their existing first-key-event acknowledgement, including the no-key-releases warning.
Keymap warnings retain their separate queue and require a new keyboard press after a frame shows the warning.
These acknowledgement exceptions do not change notice history or effective bindings.
A key press, pad press or control movement can acknowledge without consuming its normal action.
A release, MIDI clock, background completion or redraw cannot acknowledge an error.

Priority is Error, Warn, Done, Info. A lower level waits behind a visible higher level.
A new equal-level notice replaces the visible notice. A higher level interrupts it.
Keep unfinished notices in a bounded queue of 50. Preserve remaining visible time when an interrupted notice resumes.
If the pending queue fills, remove the oldest notice at its lowest level. Never remove the active error this way.
Keep all produced notices in a separate latest-50 history, including notices that expire or leave the pending queue.
Keep the existing hint as the fallback when no notice or higher-priority input prompt is visible.

No notice changes save results, retries, project state or the persistent `Save failed` indicator.
A later informational result may wait behind an error. It remains available in history.

Step 4.1 adds the message model in `crates/app/src/notices.rs`, with controlled-time tests in `crates/app/tests/notices.rs`.
The test target loads that source directly. The application does not use it until the producer migration in 4.2.
This boundary avoids temporary message levels, guessed origins and changed save feedback during model preparation.
Drawing, input dispatch and the existing one-slot messages stay unchanged in 4.1. No manual check is necessary for this step.

Keep active and pending entries in production order, with a separate active index.
Select the highest pending level, then the oldest entry at that level. Interruption does not change an entry's age.
The active entry does not count toward the pending limit. History has its own limit and stores each production once, newest first.
Equal text from two requests still creates two history entries. Reading, resuming and acknowledging entries do not create history.
The model stores the actual device label supplied by its caller. It does not discover or infer devices.

Advance the current timer before a new notice or acknowledgement. Do not queue an entry whose lifetime already ended.
An editor prompt or higher-priority state pauses visible time without stopping production or history.
After a delayed update, the next selected entry starts at that update. Do not charge it for time before it could appear.
Acknowledgement removes only the visible Error. It does not remove a timed result or change the history.
Input classification and unchanged keyboard/raw-MIDI action routing remain integration work in 4.2 and 4.3.

Review boundaries for 4.1:

- Exact deadlines and interrupted remaining time: closed for the model by `timed_notices_use_their_visible_lifetime` and `interrupted_notices_retain_remaining_time`.
- Nested interruption, equal-level replacement and original production age: closed for the model by `queued_notices_retain_priority_and_production_order` and the level-pair matrix.
- Fifty pending entries plus one active Error, with independent latest-50 history: closed for the model by `pending_notice_overflow_remains_bounded`.
- Hidden notice time, delayed updates and repeated reads: closed for the model by visibility, delayed-update, acknowledgement and history tests.
- Actual input acknowledgement, startup exceptions, save state and device routing: open for integration in 4.2 and 4.3.

The owner approved the 17-file migration plan for 4.2, including the startup acknowledgement exceptions above.
The owner then approved `crates/input/src/midi.rs` as the eighteenth file for input classification.
The inventory contains 79 one-slot producers: 53 in main, 25 in dispatch and one in saving.
Classify each result at its producer. Split mixed success/error branches instead of classifying text.
Keep recording, loading progress, input prompts, persistent save status and fallback hints outside transient lifetimes.
Loading completion clears its own state, not an unrelated queued result.
Record each startup production once. A project-read failure must not enter history twice through its old startup and flash paths.
Record all profile-load errors, not only the first error that the old slot could retain.
Keymap warnings enter history once when produced, while their own visible-warning acknowledgement remains unchanged.

Carry each controller event's port in its UI envelope. Do not infer origin from the last debug message.
Only the raw-message notification acknowledges owner input, once before its normal action. Later notifications do not acknowledge again.
Distinguish CC button release from a knob movement to zero through the actual profile and effective binding.
Keyboard repeat tracking runs before acknowledgement, including text and learning paths. Releases and modifier-only events do not acknowledge ordinary Errors.
Source metadata does not change MIDI sequencing, direct audio attempts, project stamps, velocities or original release ownership.
The shared input-status indicator, disconnection highlight and value popup remain later tasks.

Migration review boundaries:

- Mixed success/failure and delayed loading completion: closed by outcome, unrelated Error and old-project generation tests.
- Startup duplication, keymap queues and first-key exceptions: closed by startup, history-only and repeated-frame tests.
- Two devices, deferred actions and single acknowledgement: closed by raw-MIDI origin, deferred placement and shown-token tests.
- Hidden text prompts and silent autosave: closed by real filter input and existing save-compatibility tests. Task 4.3 retains dedicated save-state regressions.
- Levels and glyphs at both screen sizes: closed by drawing tests at 80×26 and 120×40.
- Text, expiry, level and keymap visibility use one frame clock, including the exact deadline.
- Load completion and project replacement clear only the matching ongoing loading state. Old-generation replies cannot reopen the previous Chop view.

### 5. Add F1 Messages without new global keys

F1 opens the existing help first. A Messages page shows newest entries first, with level and origin.
It retains the latest 50 notices in memory only. Exiting or restarting clears the history.
Tab/Shift+Tab selects Help or Messages. Up/Down and PageUp/PageDown scroll Messages. F1/Esc closes help.
History navigation does not edit pads, send sound commands or change session dirtiness.
Keep existing pad releases and the effective help keys working.

The owner approved the thirteen-file plan for step 4.4 before implementation.
Keep overlay state in `crates/app/src/help.rs`. Extend the drawing snapshot for Help and Messages.
Read the existing notice history. Do not create another queue or infer an origin from text.

Local keyboard controls act only when help receives input.
Text entry, pickers, questions and learning retain their existing priority.
Intercept these keys before the screen keymap. Do not suppress unrelated keyboard or MIDI actions.
A learned `help.toggle` action still opens and closes help. This step adds no controller history navigation.
A pad press or release retains its existing route, including a pad held before help opens.

Each opening selects Help and resets the Messages position to the newest result.
Tab and Shift+Tab switch pages. Switching pages retains the Messages position until help closes.
Up and Down move one screen row on Messages. PageUp and PageDown move one visible page.
Wrap long text and origin labels inside the message frame. Both supported sizes must permit reading every row.
Consume navigation at its ends instead of forwarding it to a parameter, bank or list.
Ignore repeated F1, Esc and page-selection presses. Repeated scroll keys can continue scrolling.
Releases do not navigate or close help.

New results enter the visible history in production order, newest first.
Keep the current screen-row offset when results arrive. Clamp it after a resize or a change of content.
Closing help does not clear history. Only a new application instance starts without previous-process entries.
Navigation, redraw and expiry add no history entry and change no saved work or sound command.
Existing non-local actions can still change the underlying screen. Closing help does not undo those actions.
The later graphics-overlay task 6.2 retains its separate kitty image checks.

Review boundaries for 4.4:

- Tab on an open pad edit: closed by the failing-then-passing `f1_navigation_stays_inside_the_overlay` regression and the five-screen navigation test.
- History at 0, 50 and 51 entries and actual origins: closed by `f1_retains_recent_results_in_order` and the App/drawing history tests.
- Long Unicode text and origin labels: closed by `long_message_text_remains_readable` at both supported sizes.
- Arrival while scrolled, terminal resize and repeated keys: closed by `f1_history_handles_empty_and_changing_content`, including an Error under help.
- Pad held across opening, page changes and closing: closed by `pad_releases_survive_f1_navigation` through keyboard, raw MIDI notes and learned buttons.
- Dirty work, project identity, undo and sound commands: closed by the history and navigation state comparisons with existing pad/undo work.
- Text, question, picker and learning priority: closed by `f1_controls_retain_input_priority_and_bindings` and `f1_keeps_settings_picker_and_learning_priority`.
- Real kitty composition, physical hardware and listening: outside step 4.4 verification. Manual acceptance remains open.

The owner reported a changing window size between Help and Messages after step 4.4.
Before correction, the drawing code centered Help around its content and gave Messages the full body.
The owner approved one stable window for both pages and a four-file correction plan.

Use the current Messages body for both pages. Keep page labels and common hints at fixed positions.
Highlight the selected page. Add the existing scroll hints only on Messages, after the shared hints.

Short help leaves unused space. Do not shrink the history to the old Help window, because that reduces readable rows.
Keep input routing, message wrapping, scroll height, sound and kitty image composition unchanged.

Stable-window review boundaries after implementation:
- Short or changed help, empty history and 50 messages: closed by `f1_pages_keep_stable_geometry` across all five screen fixtures at both catalogue sizes.
- Page labels, message count and common hints: closed by `f1_controls_keep_their_positions`, including active styling and Messages-only scroll hints.
- Terminal shrink and growth: closed by `f1_frame_follows_terminal_size`, including the last message after bounded scrolling and unchanged page height.
- Input and saved work: the existing keyboard/raw-MIDI regression suite passes. No production input or sound code changed.
- Real kitty composition, physical hardware and listening: outside this correction.

### 6. Keep value and status feedback independent

Use the existing one-second value highlight period for the shared value popup.
Show the object, parameter, formatted value and a short range bar where the parameter has a numeric range.
Only an applied change creates the popup. Pickup waiting and refused changes do not invent a value.
Use the strip's upper-right area where a strip exists, or the corresponding content area below the title on other screens.
Do not add a wave strip or move settings or FX rows just to show a value.
Keep local parameter highlights and existing limited-pad counts.
Do not move the strip or cover a text-input cursor or question.

Step 5.1 observes values before and after the existing keyboard-action and MIDI-parameter dispatch.
Compare actual state, including marked pads, rather than a requested knob position or a formatted string.
This preserves pickup, refusal, scope limits, undo and sound commands without another parameter setter.
Store the resulting object identity and value with the deadline. Selection alone does not rename the stored object.
Read the stored target again before showing its value. Undo or object replacement hides a result that no longer matches.
Session replacement clears the popup. Equal values and refused changes do not extend its deadline.
The one-second deadline includes hidden time. Existing notice timers keep their separate visible-time rules.
The UI frame removes expired snapshots, including their sample references, without waiting for another input.

Draw a bounded text panel below the title, at the right edge, without moving the existing layout.
Use a second row for numeric ranges. Categorical values have no range bar.
Pad labels include bank and key. Shared FX labels name the project slot, not the selected pad.
Buffer feedback names the buffer choice, not an unconfirmed server size. Wave feedback includes `next start`.
Keep notices, local highlights and scope-limit counts. Hide the panel during Help, editors, pickers, questions and learning.
Do not add source-status caching or kitty command handling in this step. Those remain tasks 5.2, 5.4 and 6.2.

Step 5.1 review boundaries after implementation:
- Applied versus requested values: closed by `values_appear_on_every_screen` and the pad, Chop, FX and Settings tests through production controls.
- Pickup and refusal: closed by `refused_and_limited_values_remain_truthful`, including an offline audio refusal.
- Equal repeats and exact deadlines: closed by `value_deadlines_follow_actual_changes` without sleeps.
- A selected pad at its limit or outside the marked scope: closed by keyboard and learned-button scope tests.
- Selection, screen changes, undo, sample replacement and project replacement: closed by `value_snapshots_retain_their_original_object`.
- Help, text, pickers, questions and learning: closed by `editor_prompts_take_priority_over_values` across nine states.
- Both supported sizes, Unicode and tiny terminals: closed by drawing buffer comparisons and bounded range tests.
- Expiry without input: closed by `value_popup_expiry_restores_text_without_another_input` and the sample-reference release test.
- Live kitty image composition, physical controls and listening: open, outside step 5.1 verification.

Status order is persistent save warning, recording, input, audio, project.
Input lists KB and actual connected devices. Highlight the source of a new input for one second.
Carry source metadata through existing UI routes without changing audio routing, epochs, velocity, release ownership or acknowledgements.
Device labels must remain distinguishable. An unavailable origin is not a guessed device.
Drop project, audio detail and then input detail when space is short.
Preserve recording, persistent save warnings, audio-loss warnings, nonzero xruns and the no-key-releases marker.
No transport or layer placeholder appears.

Step 5.2 prepares source state in `crates/app/src/input_sources.rs`. The visible segments remain step 5.3.
The owner approved eleven files, including this module, main, dispatch, MIDI, input tests and the six status/artifact files.
Existing MIDI envelopes carry an origin and owner-input classification, but no connection identity or receipt time.
The scanner calls `connect` before it sends Connected. The first callback can therefore precede that notification.

Give each attempted connection a process-local, monotonically increasing identity before opening it.
Keep the identity in the callback, disconnect notification and every derived UI envelope.
Capture monotonic receipt time before parsing callback bytes. Keep existing project stamps separate from source metadata.
These timestamps serve feedback only. They do not schedule sound or activate the future input-timing change 101.
Use the same receipt helper for recorded bytes. Do not create another production action route.
Do not wrap exhausted identity counters. Missing identities leave source status unknown without changing routing.

Keep only the latest connection state for each observed port: pending, connected or disconnected.
Retain disconnected generations to reject old activity and delayed connection notifications.
Input before Connected stays pending. Matching Connected can expose that input only while its original deadline remains valid.
A reconnect never inherits activity from an earlier identity. A stale source stamp cannot change the current connection state.
A newer original timestamp wins over delayed delivery. Equal timestamps retain delivery order.
Project replacement keeps device state and original deadlines. Application restart starts without previous source state.

Reuse physical activity from the same effective binding resolution: new non-modifier keyboard presses, MIDI presses and continuous movement.
For source feedback, compare continuous values within their connection, channel and control. Retain each control's original receipt time.
Old connection activity and out-of-order control values cannot seed or replace the current comparison.
Keep the existing notice-acknowledgement comparison unchanged. Source feedback has its own connection lifetime, not another binding resolver.
Do not highlight releases, inferred keyboard repeats or equal knob values. Derived MIDI events retain metadata but do not renew activity.
Read source metadata before the existing project filter. Source state does not control that filter, acknowledgements or action dispatch.
Advance source deadlines on each UI frame, including frames with no input. No saved format, binding, dependency or sound change belongs here.

Using only the last port name cannot distinguish a reconnect from queued old work.
Using UI delivery time would extend old activity and could replace a newer keyboard highlight.
Do not replace existing event channels or gate sound commands on the new source model.

Step 5.2 review boundaries after implementation:
- Two devices and keyboard: closed by `status_follows_keyboard_and_connected_inputs` on all five screens. Drawing remains in 5.3.
- Direct/deferred work, original velocities, release owners and exact deadlines: closed by `input_highlights_retain_original_receipt_time`.
- Disconnect, old activity and reconnect under the same name: closed by `disconnect_cannot_leave_a_false_active_device` and generation model tests.
- First callback before Connected, including expiry before Connected: closed by `first_input_can_precede_the_connection_notice`.
- Missing metadata: closed by `missing_source_metadata_stays_unknown`, including an unknown callback port and unchanged actions.
- Repeated values, zero movement, keyboard repeats, modifier-only input and releases: closed by production classification and lifetime tests.
- Old connections and out-of-order values: closed by two failing-then-passing `source_knob_comparison` regressions with separate channels and controls.
- Project replacement, restart and old-project payloads: closed by source lifetime and project-filter tests. Acknowledgements remain unchanged.
- Connection counter exhaustion: closed by `connection_ids_never_wrap_or_reuse_an_exhausted_counter`.
- Live controller checks and visible compact labels: open, outside step 5.2 verification.

Step 5.3 keeps the common status at the top on every screen, including screen learning.
The owner chose this layout after review of the previous learning header, which replaced the status except for save state.
The approved scope now also includes `specs/midi-learn/spec.md`. No learning action, binding or save rule changes.
The owner authorized completion of 87 and publication of its branch and PR. Merge and archive remain unauthorized.

Keep existing Status snapshots and draw entry points compatible. A separate feedback snapshot supplies sources and recording elapsed time.
The production frame passes it through one shared draw entry point, with optional graphics collection.
Build input labels from the connection model, never the last MIDI debug line.
List KB first, then connected ports in name order. Prefix each port with M1, M2 and so on within that current list.
Full labels retain the actual port. Compact labels retain distinct M numbers. An active source has a visible arrow and cyan emphasis.
Label numbers can change when the connected list changes. Match highlights by connection identity, not the label number.
Replace control characters in status labels with spaces. Keep complete original origins in message history.

Try full content first. Remove project, then optional audio details, then shorten input names and finally input detail.
Use compact spacing before dropping safety indicators. Preserve the full no-key-releases phrase.
If extreme counters compete, retain a truthful nonzero-xrun marker and REC without its elapsed detail.
No label can exceed the status row. Tiny unsupported widths can clip. Both supported sizes must retain the applicable safety indicators.
Keep recording elapsed time and source expiry independent of notices. Do not change audio work or input routing.

Learning uses the existing three-row help band: selected context, pending/refusal or layer detail, then next-step controls.
Shorten long context and detail by grapheme width. Keep the next-step controls on their own row.
The title retains its LEARN badge. The grid, notice and all screen rows keep their positions.
The existing learning lines remain available to behavior tests. Drawing uses a structured snapshot from the same learning state.

Step 5.3 review boundaries after implementation:
- Long equal prefixes, Unicode and control characters: closed by `status_width_removes_project_then_audio_then_input` and `narrow_status_retains_safety_indicators`.
- Source expiry, disconnect and reconnect: closed by `source_labels_follow_identity_when_port_numbers_change` and existing 5.2 connection tests.
- Simultaneous recording, save refusal, audio loss, nonzero xruns and no key releases: closed at both supported widths, including maximum counters.
- Pending learning, refusal, cancellation and saved assignments: closed by the production-input learning tests. Existing confirmation and clear tests remain unchanged.
- Refusal versus other-layer colour: the former uses magenta. The latter stays dim. Bounded drawing and structured snapshots retain that distinction.
- Effective learning keys and the Files exception without parameter selection: closed by `learning_footer_follows_real_controls`.
- Clear after a refused control: the same test first exposed stale refusal text. The view now follows current confirmation priority without changing learning state.
- Five screens, F1, questions and project picker: closed by drawing tests. Learning preserves all screen rows outside its help band.
- Physical controls, real kitty composition and listening: open, outside automated verification.
- Callback maximum budget: open. This check reports wall max 96.25%, thread CPU max 46.40%, and one wall sample above 50%.

Step 5.4 caches formatted content on the UI thread only.
A value-based badge key covers screen, input mode, assignment counts, marks, edit page, recording seconds and Chop toggles.
The badge formatter stays single-source. The legacy Status vector copies cached labels instead of formatting them again.
Cache connected input labels by actual connection identities and active source. Shared immutable arrays avoid copying port labels on every frame.
The status-row key covers width, save state, recording seconds, source labels/activity, backend, format, displayed CPU, xruns, master and project.
Meters, notices, value panels, playback slots and fallback hints stay outside this cache and draw from the current frame.
A new process starts empty caches. Project, screen, connection and terminal-size changes compare their current inputs, with no delayed invalidation event.
Do not cache the whole screen or add a cache to audio processing.

Step 5.4 review boundaries after implementation:
- Identical frames: closed by formatter counts and shared-array identity in cache tests. This is not an allocation-free UI claim.
- Recording seconds: closed through real keyboard/raw-MIDI recording starts and controlled elapsed time, including repeated frames within one second.
- Source highlight deadlines and reconnect identity: closed by `feedback_cache_tracks_source_identity_and_activity`, including project replacement.
- Notice/value expiry: closed by `unchanged_views_do_not_freeze_timed_feedback` through real keyboard and raw-MIDI setters, without another action.
- Meters, playback slots and fallback hints: closed by cached drawing tests at both sizes. Existing real-voice and effect-tail regressions remain active.
- Screen, edit mode and page: closed by `badge_cache_tracks_real_screen_mode_and_page_changes` through keyboard and raw MIDI.
- Status width and fields: closed by `cached_status_rebuilds_only_for_its_visible_inputs`. Displayed CPU rounding includes the half-value boundary.
- The first badge test used F3 after Files, which opens a file instead of the selected pad. Independent starting fixtures preserve that production route.
- Callback maximum budget and physical-device acceptance: open, outside this optimization.

### 7. Show only real answers

Keep each question's existing action, text, target and keyboard answers.
Resolve keyboard answer labels from effective bindings.
Show a controller answer only when its current connected binding reaches that answer through production dispatch.
Do not infer STOP or Apply from the unfinished SMC-PAD template.
If no controller answer exists, show the keyboard path. Generic `ui.confirm` remains in 88.
No new confirmation is added to direct swaps or existing safe actions.

### 8. Make rendering observable offline

`task screens` produces ANSI text from deterministic fixtures, without audio servers, MIDI devices or real project writes.
Use `<screen>-<mode>-<width>x<height>.ansi` names under `renders/screens/`.
List all registered screens and applicable existing modes in one checked catalogue matrix.
Include normal, edit pages, mark states, placements, assignment, learning, questions, recording, notices and F1 Messages where applicable.
Render both 120×40 and 80×26. A missing matrix entry fails a test.
Tests keep text inside cells and retain fixed grid, help and status rows.

The ANSI catalogue does not prove real kitty image composition.
Test image hide/restore commands separately and retain a manual kitty on/off check.
Hide or remove an intersecting image before drawing an overlay. Restore it after the overlay ends.
Cache unchanged badge and status content without freezing meters, value expiry, notice expiry or recording elapsed time.
Do not change engine code or add I/O to audio processing.

## Risks / Trade-offs

| Boundary | Required result | Status for implementation | Evidence or planned test |
| --- | --- | --- | --- |
| Save refusal followed by other messages | `Save failed` stays until a successful save | Open: integration coverage | Existing save-state tests plus new priority tests |
| Error followed by release or background input | Error stays visible | Closed for notice migration | Real keyboard/raw-MIDI releases, repeats, zero movement, background loading and identical replacement tests |
| Simultaneous file, copy and swap | Show pending modes without changing source-specific priority | Closed for title step | `overlapping_placements_remain_visible_without_changing_priority` covers all seven combinations with keyboard and raw MIDI |
| Occupied swap target | No invented replacement question | Closed for title step | The same production-input test checks direct swaps and remaining pending file placement |
| Copy cancellation or loading, same-file placement, loading swap endpoints | Do not invent a target or confirmation | Closed for pad-state step | `pad_corner_marks_respect_loading_cancellation_and_whole_file_identity` checks snapshots and real input results |
| Pad states beside a corner mark | Keep key, name, bank, hit, selection and marked fill readable | Closed for pad-state step | `pad_corner_marks_keep_identity_and_state_styles` covers 13, 24 and 40 cell widths, Unicode names and slice 128 |
| Assignment followed by bank change or cancellation | Numbers follow the actual plan. Pads stay unchanged. Slice audition remains | Closed for pad-state step | `assignment_numbers_appear_on_target_pads` checks by-hand and ordered plans through keyboard, MIDI notes and buttons |
| Mode starts with the grid hidden | Keep the hidden layout. Assignment temporarily shows the grid | Closed for pad-state step | `learning_and_hidden_grids_keep_existing_sound_rules`, `yellow_grid_feedback_keeps_shared_geometry_and_hidden_layout` at 80×26 and 120×40 |
| Queued and interrupted notices | Bounded storage and deterministic remaining time | Closed for notice migration | Twenty-one controlled-time model tests plus production input, frame-clock and loading tests |
| Disconnect and stale source event | No false connected-device label or stuck highlight | Open: new coverage | Production multi-device and reconnect sequences |
| Short or hidden-grid screen | Keep save warning, controls and mode identity | Open: new coverage | 80×26 and hidden-grid fixtures |
| Overlay expires without another press | Restore wave image and hint | Open: new coverage | Timer-driven graphics and buffer tests |
| Source metadata travels with input | Audio routing and original releases stay unchanged | Open: integration coverage | Existing parity, pending-order and epoch tests |
| UI caches unchanged content | Recording, levels and timers still advance | Open: new coverage | Controlled-time view tests |
| Long Unicode filename, variants and 128 slices | Keep screen, PadMode, variant and slice count inside the title | Closed for title step | `narrow_titles_retain_the_pad_mode` at 80×26 and 120×40, including `chop 128 of 129` |
| MIDI list scrolls in Settings | Keep LEARN, selected tab and selected row visible | Closed for title step | `settings_title_badges_stay_visible_while_midi_rows_scroll` |
| Playback changes beside mode badges | Keep ten-cell slots, styles and following columns | Closed for title step | `playback_slots_remain_stable_beside_modes`, existing current-voice and stale-project tests |

## Owner review decisions

The owner approved these choices after the specification review:

- F1 page controls: Tab/Shift+Tab selects Help or Messages. Up/Down and PageUp/PageDown scroll Messages. F1/Esc closes help.
- Notice queue: equal levels replace. Interrupted notices retain remaining visible time. Overflow removes the oldest lowest-level pending notice.
- Overlapping placements: one compound badge names all pending placement modes. Existing keyboard/MIDI differences stay unchanged.
- Catalogue delivery: `task screens` produces local ANSI files. Do not commit a large generated catalogue before its implementation review.
- Compact edit badges retain their page at 80 columns: `EDIT BOUNDS`, `EDIT FX 2` or `EDIT FILTER`, not only `EDIT`.

This approval implies no hardware result, listening result or callback maximum-budget acceptance.

## Open questions

None for preparation, title badges, pad feedback or the step 4.1 model. The owner approved each file plan before implementation.
Ask the owner before changing an approved choice.

### Step 4.2 input classification scope

The owner approved the eighteenth file, `crates/input/src/midi.rs`, before implementation.

`Profile::is_knob` checks the profile's physical CC controls, not learned assignments.
An unnamed CC learned as a command still passes that check as a knob.
The existing resolution discards a non-pad command release. A knob at zero can also have no action on the current screen.
Checking only the number zero or `is_knob` can therefore acknowledge an Error on a button release.

Recommended: expose control classification from the same effective binding resolution that produces the action.
Keep the existing action results and all callers' routing behavior unchanged.
Test learned CC button press/release and continuous CC movement to zero, including repeated values and separate devices.
Do not duplicate binding precedence in application code or resolve a synthetic press to classify an actual release.

Decision: expose classification from the existing resolution, with the original action result unchanged.
A declared physical control keeps its type. For unnamed CC controls, the effective learned command identifies a button and a parameter identifies a knob.
An unassigned unnamed CC keeps the existing knob interpretation. This rule makes no claim about unmeasured hardware.

Only an Error token captured by a shown frame can acknowledge that exact production.
A replaced Error, including identical text, needs its own shown frame. Pending input cannot acknowledge a notice that never appeared.
Keep token identity stable across queue removal and interruption. Controlled-time token tests and production keyboard/raw-MIDI tests close this boundary.

### Step 4.4 scrolling default

Should a scrolled history keep the same message anchored when a new result arrives?
Default: retain its screen-row offset and clamp it to the current content.
New results remain live and newest first. Stable message anchoring is not part of this step.
