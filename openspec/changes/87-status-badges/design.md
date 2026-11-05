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
| Pad or slice edit | `EDIT · <page>` | `EDIT` |
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

### 3. Add marks without changing pad actions

The owner approved yellow for modes where pads do more than ordinary playback.
Marking still sounds. Learning and questions retain their existing routing.
A yellow grid outline does not replace bank membership, white hit flashes or the selected-pad indication.
When the grid is hidden, keep the mode badge and existing pad line. Assignment retains its current forced-grid rule.

Use `◎` for available file/copy targets and `!` for occupied targets that already require confirmation.
Use `◎` for swap targets without implying a new question: occupied swaps remain direct.
Show planned slice numbers during assignment. Existing mark dots retain their positions.
Never use color as the only distinction between copy, swap, file placement and learning.

### 4. Centralize transient notices

Use Done, Info, Warn and Error levels with Keyboard, Device or System origins.
Route existing one-slot messages and startup notices through this model.
Keep persistent save state, recording state, loading progress and editor prompts separate from transient lifetime rules.
A frame does not generate another history entry for the same state.

Done and Info last three seconds of visible time. Warn lasts six seconds.
Manual save success keeps five seconds. Error waits for the next new owner action.
The startup no-key-releases warning retains its existing first-keypress acknowledgement rule.
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

### 5. Add F1 Messages without new global keys

F1 opens the existing help first. A Messages page shows newest entries first, with level and origin.
It retains the latest 50 notices in memory only. Exiting or restarting clears the history.
Tab/Shift+Tab selects Help or Messages. Up/Down and PageUp/PageDown scroll Messages. F1/Esc closes help.
History navigation does not edit pads, send sound commands or change session dirtiness.
Keep existing pad releases and the effective help keys working.

### 6. Keep value and status feedback independent

Use the existing one-second value highlight period for the shared value popup.
Show the object, parameter, formatted value and a short range bar where the parameter has a numeric range.
Only an applied change creates the popup. Pickup waiting and refused changes do not invent a value.
Use the strip's upper-right area where a strip exists, or the corresponding content area below the title on other screens.
Do not add a wave strip or move settings or FX rows just to show a value.
Keep local parameter highlights and existing limited-pad counts.
Do not move the strip or cover a text-input cursor or question.

Status order is persistent save warning, recording, input, audio, project.
Input lists KB and actual connected devices. Highlight the source of a new input for one second.
Carry source metadata through existing UI routes without changing audio routing, epochs, velocity, release ownership or acknowledgements.
Device labels must remain distinguishable. An unavailable origin is not a guessed device.
Drop project, audio detail and then input detail when space is short.
Preserve recording, persistent save warnings, audio-loss warnings, nonzero xruns and the no-key-releases marker.
No transport or layer placeholder appears.

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
| Error followed by release or background input | Error stays visible | Open: new coverage | Controlled keyboard/raw-MIDI sequences |
| Simultaneous file, copy and swap | Show pending modes without changing source-specific priority | Open: integration coverage | `modes_85_overlapping_hands_keep_source_specific_priority` |
| Occupied swap target | No invented replacement question | Open: integration coverage | Recorded swap and question tests |
| Queued and interrupted notices | Bounded storage and deterministic remaining time | Open: new coverage | Controlled-clock queue and overflow tests |
| Disconnect and stale source event | No false connected-device label or stuck highlight | Open: new coverage | Production multi-device and reconnect sequences |
| Short or hidden-grid screen | Keep save warning, controls and mode identity | Open: new coverage | 80×26 and hidden-grid fixtures |
| Overlay expires without another press | Restore wave image and hint | Open: new coverage | Timer-driven graphics and buffer tests |
| Source metadata travels with input | Audio routing and original releases stay unchanged | Open: integration coverage | Existing parity, pending-order and epoch tests |
| UI caches unchanged content | Recording, levels and timers still advance | Open: new coverage | Controlled-time view tests |

## Owner review decisions

The owner approved these choices after the specification review:

- F1 page controls: Tab/Shift+Tab selects Help or Messages. Up/Down and PageUp/PageDown scroll Messages. F1/Esc closes help.
- Notice queue: equal levels replace. Interrupted notices retain remaining visible time. Overflow removes the oldest lowest-level pending notice.
- Overlapping placements: one compound badge names all pending placement modes. Existing keyboard/MIDI differences stay unchanged.
- Catalogue delivery: `task screens` produces local ANSI files. Do not commit a large generated catalogue before its implementation review.

This approval implies no hardware result, listening result or callback maximum-budget acceptance.

## Open questions

None for the preparation step. Ask the owner before changing an approved choice.
