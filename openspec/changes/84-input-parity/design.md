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
not fixes for all differences.
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

### Tests before completion

Run the shared workflows on Pads, pad edit, marking, file in hand, pad swap, pad copy
(from 83a), question, learn, Chop before assignment, linked-slice listen and by-hand assignment. Keep existing
X-Session tests unchanged. Add gate release coverage after a screen and bank change.
A difference outside the Chop, copy and approved file/swap/question deltas blocks the
affected step until it is documented and its scope is agreed. Do not skip the test or
change an assertion to accept an unknown result.

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
- The five-atomic snapshot and the pending-command race remain work for 85. The
  findings above use separate published frames; they do not authorize a same-frame
  race fix.

## Risks / Trade-offs

- A helper that feeds resolved actions directly can pass while production MIDI fails.
  Test raw bytes through dispatch and App, including profile initialization.
- Adding velocity to an internal event affects senders, App matches and test fixtures.
  Find every `Event::PadHit` use; keep queue-acceptance behavior and release tracking.
- App handles Chop on the UI thread today. Preserve that route; do not move ordinary
  performance pad hits onto it or promise lower Chop latency.

## Open questions

- Will further paired tests expose a difference outside the approved repairs?
  Default: record it and stop the affected task for another scope decision. The new
  approval covers only the three confirmed file/swap/question differences above.
- Which physical pad controller and preset will the owner use for acceptance? Default:
  the existing notes 36–51 fallback or an explicit learned pad binding. No SMC-PAD
  relative mode or port assumption is needed for 84.
