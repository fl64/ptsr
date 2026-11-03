## Context

Change 84 is active after keymap cleanup 83a. The owner authorized this work on
2026-11-03 and requested the existing branch `plan/controller-first-patterns`.
This is an explicit exception to the separate-branch rule. No preparation merge is
needed before 84. Push and PR creation need a separate owner request.

The implementation baseline is `2b084d1`: 83a was accepted and archived through
`openspec archive` in `openspec/changes/archive/2026-11-03-83a-keymap-cleanup/`.
The owner accepted 83a without completing the manual checks. This is not evidence of
sound heard by the owner or of physical SMC-PAD tests. Automated `task check` passed
at that baseline (559 App tests and 69 input tests). Change 82 is also archived.

Task 0.1 rechecked these source locations at `2b084d1`:

- `crates/app/src/main.rs:5637`: `App::keyboard_event` is the production keyboard
  entry point, including text, learn and held-key guards. Use it in the paired driver.
- `crates/app/src/main.rs:1558` and `:2792`: keyboard Chop pad presses call `chop_pad`.
  It handles slice pages, linked pad sounds and selection, and assign-plan targets.
- `crates/app/src/main.rs:4633`: the MIDI Chop branch still calls `play_slice` instead.
  The question guard is before it; the screen-learn guard is at `:4574`.
- `crates/app/src/main.rs:4640`: the MIDI pad-copy route from 83a is present. Include
  copy in the baseline tests; keep J/L values, U/O banks and Tab assign order.
- `crates/app/src/midi.rs:35`: `Event::PadHit` carries the absolute pad and audio queue
  acceptance, but no hit velocity. Both send sites are at `:431` and `:455`.
- `crates/app/src/midi.rs:286`: `dispatch` resolves profiles and tracks held pads.
  Its test helper `dispatched` at `:928` does not drive App or profile seeding by App.
- `crates/input/src/midi.rs:620`: `Profile::resolve_in` preserves note velocity and
  resolves edit and learned bindings. `Msg::parse` at `:29` accepts raw MIDI bytes.
- `crates/app/src/main.rs:9107` and `:9766`: `test_app` and `NoticeInput` provide the
  App fixture and keyboard held-state helper. The shared MIDI state is published in
  the main loop at `:6073`; re-use that production publication in the driver.

Do not infer hardware support from the tests. The SMC-PAD profile is still a template.
The X-Session Pro has no pads; use its shipped profile for regression and explicit
learned pad bindings, not as an invented physical pad device.

## Goals / Non-Goals

Prove equivalent pad workflows through the two real input paths and fix the confirmed
Chop difference. Keep the keyboard behavior, MIDI velocity, releases, project epochs,
queue acceptance and X-Session safe defaults. Do not implement the later input model,
controller profile or sequencer here.

The broad parity goal covers all screens and modes in the roadmap. This delta specifies
Chop and the owner-approved MIDI copy, file/swap target and question-route repairs,
plus the separately approved held-keyboard release through learn and pad playback
line, not fixes for all differences.
A newly found difference is a scope decision, not permission to change more behavior.

## Decisions

### One test driver, two input sources

Build a test-only `Drive` around the existing App test fixture. It supplies key press and
release events, raw MIDI bytes with a port identity, and a deterministic `frame` boundary.
Use existing dispatch helpers where possible. If extraction is needed, move only the
routing body into a shared production function in a separate behavior-neutral commit.
Do not replace real resolution with a test-only interpretation of the profile.

The MIDI route is byte parse -> actual profile resolver and seeding -> production MIDI
dispatch -> stamped UI events -> App handling. Capture the direct engine queue as well
as App commands. Drain and publish state at explicit boundaries, without timing sleeps.
Keep source channel, original held-pad identity and project epoch checks.

Use the shipped X-Session TOML and a small test pad profile (notes 36–51 and buttons on
CC25–29). The test profile is not a new shipped SMC-PAD map. The keyboard and MIDI passes
use matching velocities (Shift and MIDI 127) for equality tests. The shipped plain
keyboard velocity is 80, not 100. Test lower MIDI velocities
separately; the keyboard is not required to produce every MIDI velocity.

Compare commands in order and the visible state: selected pad and slice, bank, plan,
in-hand object, marks and notices. Compare meaning, not input-debug lines that name the
source device. Assert that no extra direct pad hit accompanies a UI preview or assignment.

### Small Chop routing fix

Carry the original velocity in `Event::PadHit` and preserve it at both dispatch send sites.
Call `chop_pad` from the MIDI Chop branch with the pad's bank-local position and velocity.
Check the published bank and the existing same-frame behavior; do not add the future
acknowledgement protocol in 84. A new race finding goes into the divergences below.

Chop remains on the UI route. Normal performance pads still go directly to the audio
queue; the UI must not trigger them again. Keep the `accepted` field and epoch filtering.
Keep question and learn guards before the Chop handler. Assignment previews follow the
existing keyboard logic; do not add a new velocity rule for bare slice previews.

### Linked Chop gate release (owner approved on 2026-11-03)

The owner approved the narrow release repair after the four failing task 2.1 gate tests.
Keep the original device/channel/control, absolute pad and project epoch for UI-routed
Chop hit candidates, separate from accepted direct hits. Give a candidate a stable
press identity until its control releases or disconnects. Carry that identity to App;
only a Chop handler call whose pad command enters the UI audio queue records a pad hold.
Bare previews, assignment targets, guarded and queue-rejected hits do not become holds.
Return the local send result to the Chop handler; do not send an acknowledgement to MIDI.
Record the audio-held epoch only after a successful push. A rejected retrigger must keep
an older accepted hold and its release; a rejected hit from another control must not
acquire that hold. Buffer an accepted hold's release if the UI audio queue is full.
Chop tests use offline Audio with its real project gate, not an invented send result.

On release, dispatch reports the original UI candidate and epoch without sending a
second direct audio release. App releases a recorded pad hold through its existing
project-scoped audio sender, before question/learn hit guards can consume it. Accepted
direct hits retain their current queue, pending-release and acceptance rules. If one
control has both direct and UI holds for the same pad/epoch, report the direct release
first and clear the matching UI hold so it does not release twice. Project replacement
clears App's UI hold records; stale stamped events remain rejected by the existing gate.
This is local hold bookkeeping, not feedback to MIDI or 85's acknowledgement protocol.

Cover note-off and note-on zero, CC0, disconnect, source/channel separation, mode guards,
route changes and project epochs. Use offline Audio to check the UI gate actually stops;
no owner audio device is opened. The full parity WAV remains task 2.2.

### Held keyboard release through learn (owner approved on 2026-11-03)

After the keyboard-only gap below was reported separately, the owner approved its narrow
repair with "go" in the continuation session. A keyboard pad held before F10 must pass
its release through screen learn. Use the existing held-key filter and Pads' original
absolute target; bank or screen changes before F10 must not move the release target.
Only a pad key already tracked as held may pass. Add a read-only held-pad query to the
input helper so a new silent learn-mode key cannot release a pad held by MIDI.
Do not pass new hits, other commands or unrelated releases through the learn guard.
Keep learn picks, pending controls, Enter, Esc and saved bindings unchanged. A release
must not pick another element or confirm a binding. Test the keyboard, raw notes and
learned CC80 button with published frames, plus a silent new key beside an older hold.
No change to text input, picker priority or the 85 pending-action protocol is authorized.

### Silent copy targeting (owner approved on 2026-11-03)

The owner approved the copy-route repair after the source audit below. Publish a copy
in hand as a UI-only pad mode, as file-in-hand and swap already are. Do not send a direct
pad hit while a copy or its replacement question is open. Select the same target as the
keyboard before showing the question, so Enter acts on the named pad. Keep existing
cancellation and original held-pad releases. This restores the 83a contract; it adds no
new copy command or question policy. Implement it separately from the Chop fix.

First extract the existing shared-state publication unchanged for production and tests.
Then build the driver around production keyboard events, MIDI dispatch and App seeding.
The source-only finding must get a failing raw-MIDI regression before the repair.

### File, swap and question parity (owner approved on 2026-11-03)

After the six failing task 1.2 tests below, the owner explicitly approved adding these
three repairs to 84. This approval is recorded before repair code. It is separate from
and does not follow implicitly from the earlier copy approval.

- A file-in-hand MIDI target selects the same bank-local pad as the keyboard. Preserve
  file loading, overwrite questions, Esc and Enter; do not change load-completion undo.
- A MIDI swap selects the same target and records the same one-step undo as a keyboard
  swap on Pads. Keep empty targets, same-source cancellation and loading refusal.
- Publish an open question as a UI-only pad state. The existing App guard consumes
  new pad hits without sound or state changes. Keep the dispatch release path before
  the hand/learn guards so a pad held before the question still releases its original
  identity. Closing the question restores the normal direct MIDI performance route
  only when no other UI-only pad mode remains. A file or copy still in hand keeps its
  existing silent target route.

Do not change confirmation commands, marking sound, the five-atomic snapshot or the
pending-command protocol. Test keyboard, notes and explicitly learned pad buttons at
published frame boundaries. Further differences still require an owner scope decision.

### Pad playback line (owner approved during acceptance)

The owner reported a missing green line while playing a slice from a pad key in the
open Chop editor. They clarified that Space already shows the green moving line.
The final agreement is only to show that line for pad playback too. The earlier
suggestion to change M was withdrawn. Space, M, cursor control, slice selection and
the sound keep their existing behavior. This agreement does not change the main
spec's existing cut rules or resolve wording differences between older requirements.

At `efea1e0`, `App::playhead` reads only `preview_frame` (`main.rs:2742–2746`),
and the engine publishes only a voice marked `preview` (`engine/src/lib.rs:965–972`).
A linked Chop hit starts a real pad through `Pads::apply` (`main.rs:2824–2827`), not
that preview voice. The waveform already draws a green line when given a position
(`tui/src/lib.rs:2430–2433`). This source path explains the missing position for linked
pad hits; it is not a live agent check. The owner has identified the pad trigger path.

Implement read-only audio position reporting and pass a matching pad's position to
the waveform. Do not start an extra preview, change an audio route or substitute a
UI timer for the real position. Keep a separate display position from the preview
position used by transport and edits: adding the line must not change M or Space.
Playback progress must not drag the owner's cursor. Keep existing hit-driven slice
selection; this is not permission to remove that selection behavior. No new mode,
key, header behavior, sound algorithm or project/config format is approved.

Test keyboard events, raw notes and learned CC through the real offline audio queues.
Check the real frame position, pitch, loop, end/release, source identity, project
changes, queue rejection and question/learn guards. Check that the new line does not
change the existing cut or transport command, and that the sound is not duplicated.
Cover the line in the existing waveform modes and keep the original Space tests.
Update the owner checks and release build only after the implementation passes.

The owner also reported that no controller is currently available. The physical MIDI
pass remains unverified. No test result or acceptance is inferred from its absence.
The keyboard report is not acceptance of the rest of 84, and archive remains blocked.

### Tests before completion

Run the shared workflows on Pads, pad edit, marking, file in hand, pad swap, pad copy
(from 83a), question, learn, Chop before assignment, linked-slice listen and by-hand assignment. Keep existing
X-Session tests unchanged. Add gate release coverage after a screen and bank change.
A difference outside the Chop, copy, approved file/swap/question, held-keyboard learn
and pad playback line deltas blocks the affected step until it is documented and its scope is agreed.
Do not skip the test or change an assertion to accept an unknown result.

Render paired linked-pad hits and low/high MIDI velocity through the implemented route
to `renders/84-input-parity.wav`. Check finite samples, no clipping, expected silence,
relative levels and the existing nearest-block timing. Do not claim input timestamps or
new latency guarantees. New scenario tests use the exact scenario names in the delta.
No scenario goes into the main specs until its test passes.

## Found divergences

- Confirmed at the baseline: MIDI Chop pads bypass `chop_pad`. This loses the keyboard
  slice selection and assign-target behavior and uses only the local slice number.
  Fix in 84; preserve original velocity for the linked-pad path.
- Source audit after task 0.1 found a separate copy-route gap at `2b084d1` (unchanged
  by the activation commit). In `crates/app/src/main.rs:6084`, `shared.hand` includes
  file-in-hand, swap and Chop, but not `copy_in_hand`. On Pads with a copy in hand,
  `crates/app/src/midi.rs:438-455` therefore sends a ready hit to the direct audio
  queue before App handles the copy. An occupied loaded target can sound before the
  replacement question. This contradicts the existing main-spec scenario
  "Copy target from a controller in another bank" in `pad-assignment`.
  The test at `crates/app/src/main.rs:20334` injects a ready `Event::PadHit` and
  checks only App commands; it cannot detect that direct-queue hit.
  This is a code-path finding, not a live sound or hardware result. The owner approved
  an explicit copy-route delta and separate repair on 2026-11-03. The driver and raw-MIDI
  regressions come first. No fix was included in task 0.1.
- Task 1.2 at `a237591` confirmed three more differences through keyboard events,
  raw notes and an explicitly learned CC80 pad button. Before repair, all six tests
  failed. The owner then approved the narrow repair described in Decisions. The red log is
  `/tmp/ptsr-84-1.2-baseline.log` (9 passed, 6 failed). Existing copy tests still pass.
  - `baseline_swap_parity_from_raw_notes` and
    `baseline_swap_parity_from_learned_button`: take pad 1 with Ctrl+X, turn to bank 2
    and hit local pad 3. Both inputs swap absolute pads 0 and 18 without a hit sound.
    Keyboard selects absolute pad 18 and records one undo step; both MIDI paths leave
    selection at absolute pad 16 and record no undo step. The MIDI branch calls
    `swap_with` directly (`main.rs:4671`), without the keyboard target selection or
    `apply` history wrapper (`main.rs:897`, `:1930`, `:5280`).
  - `baseline_file_hand_parity_from_raw_notes` and
    `baseline_file_hand_parity_from_learned_button`: a file is in hand, bank 2 is open
    and local pad 3 holds another sound. All paths stay silent and ask about absolute
    target 18. Keyboard selects that target; both MIDI paths leave selection at 16.
    The MIDI branch calls `put_in_hand` directly (`main.rs:4660`); the keyboard branch
    also sets selection (`main.rs:2134`). This test does not decode a file or prove
    load-completion undo behavior.
  - `baseline_question_keeps_pads_silent_from_raw_notes` and
    `baseline_question_keeps_pads_silent_from_learned_button`: mark two loaded pads,
    turn to bank 2, press Backspace to open the clear question, then hit loaded local
    pad 3. Keyboard sends no hit. Both MIDI paths send a direct
    `PadPress { pad: 18, velocity: 1.0 }` before App's question guard
    (`main.rs:4656`). `publish_midi` does not publish a general question as a UI-only
    pad state (`main.rs:4489`), so dispatch uses its direct branch (`midi.rs:493`).
    This is command-queue evidence, not a listening or hardware result.
  Pads, pad edit, sounding marking across banks and silent screen learn pass the
  paired baseline tests. Tasks 1.2 and 1.4 stay open until the approved repairs and
  regressions pass. Do not remove, skip or weaken the failing tests to complete them.
- Task 2.1 draft on `4f6c1be` carries velocity and calls `chop_pad`. All six Chop
  scenario tests pass at the command/state level. This also exposes a release gap
  for gate pads now started by the UI route. The draft is not ready to commit.
  `/tmp/ptsr-84-2.1-route.log` has 6 passed and 4 failed:
  - `chop_linked_gate_release_reaches_audio` and
    `chop_linked_button_gate_release_reaches_audio`: hit linked gate pad 18 (zero-based)
    on Chop in bank 2, then send note-off or CC80=0. Keyboard sends one
    `PadRelease { pad: 18 }`. Both MIDI paths send no audio command. The UI held flag
    clears in the same bank, but the newly started gate voice gets no release.
  - `chop_linked_gate_release_keeps_original_pad` and
    `chop_linked_button_gate_release_keeps_original_pad`: after that hit, turn to bank 3,
    open FX with F4 and release the original control. Keyboard still releases pad 18.
    Both MIDI paths send no audio command and leave the original UI held flag true.
  The MIDI hand branch (`midi.rs:484`) sends `PadHit` without recording a held identity;
  only an accepted direct hit enters `pressed` (`midi.rs:508`). An untracked release
  uses the current bank (`midi.rs:523`). App's `PadRelease` branch only clears UI held
  state (`main.rs:4690`), because the old direct path had already sent the audio release.
  A linked Chop hit now sends its pad voice through App, so that old release rule is
  not sufficient. At the committed baseline MIDI Chop played a slice preview, not that
  gate voice; this is a regression risk exposed by the proposed route, not a claim of
  a heard or physical-controller result. The owner approved the narrow release extension
  on 2026-11-03. Keep all four regressions and implement the decision above before task
  2.1 can pass.
- The release draft now passes its MIDI gate, queue-full retrigger, route/epoch,
  disconnect, device/channel/control and offline gate-stop checks. A broader silent-hit
  regression exposed an existing keyboard-only learn release gap. In
  `chop_silent_hits_have_no_pad_release`, hold keyboard pad 18 before opening screen
  learn with F10, send and release an ignored MIDI control, then release the original
  keyboard key while learn stays open. The ignored MIDI control sends no pad hit or
  release and preserves the older held state. The keyboard release then sends no
  audio command: `keyboard_event` calls `learn_key` before `HeldPads::filter`, and
  `learn_key` returns immediately for every `KeyEventKind::Release` (`main.rs:3806`,
  `:5706` before further edits). This early return is present in `4f6c1be`, not added
  by the MIDI repair. This is command/state evidence, not a heard gate result.
  `/tmp/ptsr-84-2.1-current-check.log` has 595 passed and 1 failed; the targeted Chop
  suite has 18 passed and 1 failed. Formatting and clippy pass. No test was removed,
  skipped or changed to accept the missing keyboard release. The owner then approved
  the narrow keyboard repair in the continuation session; see Decisions above. Keep
  task 2.1 open until that repair and all regressions pass the full check.
- The five-atomic snapshot and the pending-command race remain work for 85. The
  findings above use separate published frames; they do not authorize a same-frame
  race fix.

## Task 2.1 verification

After the separate keyboard approval, `held_pad_release_passes_screen_learn` reproduced
`release 18: []` before the keyboard fix (`/tmp/ptsr-84-keyboard-learn-red.log`). The
repair keeps that regression and `chop_silent_hits_have_no_pad_release` unchanged.
`held_pad_release_passes_screen_learn` and `new_learn_keys_do_not_release_an_older_pad`
cover the new delta through production keyboard, raw notes and learned CC80 routes.
The offline gate-stop test also covers a bank/screen change followed by F10 for all
three sources. Tests verify commands, learner state and offline samples, not hardware.

`/tmp/ptsr-84-2.1-approved-check.log` records a green full `task check`: formatting,
strict clippy, workspace tests (598 App tests, 70 input tests, no failures) and the
callback benchmark. The 32-voice callback mean is 9.32% and p99 is 16.12% of the block
budget; its maximum is 111.34%. Strict OpenSpec validation passes all 16 items.
At that point task 2.2 and owner acceptance were still open, and the full parity WAV
was not present. Task 2.2 is recorded below; owner acceptance is still open.

## Task 2.2 verification

Commit `9f45ae4` adds five offline route tests in `crates/app/src/input_tests.rs`.
`Drive::offline` connects production MIDI dispatch directly to the MIDI producer from
`audio::offline`; App uses that Audio's UI producer and project state. Both queues feed
the real `Realtime::process`. No direct command is captured and replayed for these tests.
The timestamped engine render is a separate comparison for one expected hit and release,
not a substitute for the input route. Production code is unchanged in this step.

The tests cover keyboard, raw note and learned CC80 hits; matched velocities 80 and 127;
pitch +12 and low-pass at 1200 Hz; and low/high velocities 23 and 101. Separate controls
with pitch 0 or filter off must change the samples. Note-off, note-on zero, CC0 and
disconnect release the original absolute pad after a bank change, Files and F10 learn.
The direct MIDI test computes the first block before App receives the event, then checks
that App adds no second hit or release. The full-queue test uses the real 1024-slot UI
queue: a rejected retrigger does not reset the older voice's phase, and its pending
release reaches audio when space returns. Project tests leave old hit/release commands
in each real queue and deliver old UI events after a new gate starts. Old and not-ready
hits stay silent; an old release does not stop the new gate. Additional renders compare
bare previews and assignment and check silence under questions and learn.

`renders/84-input-parity.wav` is stereo float32 at 48000 Hz, 258048 frames (5.376 s).
It contains seven segments of 0.768 s, each with a hit at local frame 4096 and release
at 20480. The source slice would last 0.5 s at pitch +12, so post-release silence proves
the gate stopped before the source ended. Listen in this order:

- 0.000–2.304 s: keyboard, raw note, learned CC80; three identical full-strength hits.
- 2.304–3.840 s: raw note at velocity 23, then 101.
- 3.840–5.376 s: learned CC80 at velocity 23, then 101.

All matched routes are sample-identical. Low/high peaks are 0.025652 / 0.112647,
ratio 0.227723. An independent read of the saved WAV confirms finite samples, peak
0.141645, maximum adjacent-sample step 0.009306, expected silence and all seven onsets
at local frame 4097 (one sample after the block boundary). This checks existing block
routing, not input timestamps, physical latency or a new latency guarantee.
SHA-256: `f795d105ad3f098d03857203e0eaffed73e782dd1d51bcfdfec233f7d35a4da6`.

Recreate the ignored local WAV with
`cargo test -p ptsr input_tests::render_84_input_parity -- --nocapture`.
As with other renders, `/renders/*.wav` is ignored by git; this step does not change
that delivery rule. The file is present in the owner's local worktree, not in the commit.

`/tmp/ptsr-84-2.2-final-check.log` records a green full `task check`: formatting,
strict clippy, workspace tests (603 App and 70 input tests, no failures) and benchmark.
`/tmp/ptsr-84-2.2-paired.log` has 43 paired-input tests passed, no failures.
That 32-voice callback run uses mean 8.43%, p99 10.24%, maximum 25.47% of the block
budget. The subsequent full completion check also passes
(`/tmp/ptsr-84-2.2-completion-check.log`), with mean 9.71%, p99 15.61% and maximum
51.55%. The last maximum exceeds the 50% target; a green command does not prove a
hard maximum below that target. This test-only step changes no callback code and does
not resolve the benchmark variability already recorded in task 2.1.
Strict OpenSpec validation passes all 16 items. No live, hardware or listening check
was made. At that point release build, user documentation and owner acceptance were
still open. Task 3.1 is recorded below; owner acceptance and archive remain open.

## Task 3.1 verification

`docs/USER-STORIES.md` now gives separate keyboard and controller passes for Chop,
including preview, linked-pad settings, velocity, assignment and original gate release
through learn. It also describes silent copy, file and swap targets and undo. The
assignment instructions follow the current selected-slice rule and no longer list J/L
slice selection as an ignored command. These are documentation corrections to existing
specs and code, not new behavior.

Section 84 of `docs/LIVE-CHECKS.md` has 21 unchecked owner checks, with separate input
passes, a long-slice gate setup and the seven-hit WAV listening order. The device,
preset, terminal, available MIDI release forms and listening result are explicitly
unverified. The X-Session has no physical pad grid; the SMC-PAD profile remains work
for 91. Automated MIDI results are not hardware acceptance.

`/tmp/ptsr-84-3.1-check.log` records a green full `task check`: format check, strict
clippy, workspace tests (603 App, 70 input and 98 TUI tests, no failures), scenario
coverage and callback benchmark. The 32-voice run reports mean 9.11%, p99 12.15%,
maximum 24.10% of the block budget. This does not erase the earlier maximum above 50%.
Strict OpenSpec validation passes all 16 items, and `git diff --check` is clean.
`task build` runs `cargo build --release -p ptsr` and passes; its output is in
`/tmp/ptsr-84-3.1-build.log`. The release binary is `target/release/ptsr`.
No program was started, no audio device was opened and no owner config was changed.
The agent has not heard the WAV. Task 3.2 needs the owner's results; task 3.3 still
needs explicit acceptance before the OpenSpec CLI archive.

## Risks / Trade-offs

- A helper that feeds resolved actions directly can pass while production MIDI fails.
  Test raw bytes through dispatch and App, including profile initialization.
- Adding velocity to an internal event affects senders, App matches and test fixtures.
  Find every `Event::PadHit` use; keep queue-acceptance behavior and release tracking.
- App handles Chop on the UI thread today. Preserve that route; do not move ordinary
  performance pad hits onto it or promise lower Chop latency.

## Open questions

- Will further paired tests expose a difference outside the approved repairs?
  Default: record it and stop the affected task for another scope decision.
- Which physical pad controller and preset will the owner use for acceptance? The
  owner has no controller at present. Default: keep that pass unverified and use the
  existing notes 36–51 fallback or an explicit learned binding for automated tests.
  No SMC-PAD relative mode or port assumption is needed for 84.
- Which line should win if a Space preview and several matching pad voices sound
  together? Not explicitly decided by the owner. Proposed default: preserve the
  existing preview line while a preview sounds; otherwise show the most recently
  started sounding pad of this chop. Do not add several lines or change which voices
  sound. Recheck this default before implementing the display selection.
