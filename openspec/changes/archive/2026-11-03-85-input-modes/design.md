## Context

See `proposal.md` for the purpose and owner checks. Baseline: `f52cf00` in
`/home/user/git/fl64/ptsr`, branch `plan/controller-first-patterns`.
The owner reviewed these artifacts and approved implementation with separate
signed-off commits after green checks on 2026-11-03. Continue on
`plan/controller-first-patterns` with the agreed narrow scope and preserved source
boundaries. This approval does not authorize push, PR, archive, live audio or a
physical-controller acceptance claim.

Source audit at that baseline:

| Source | Existing boundary |
| --- | --- |
| `crates/app/src/main.rs:4541–4561` | Publishes bank, screen, edit, F10 learn and the combined hand/question/Chop route separately. |
| `crates/app/src/midi.rs:480–605` | Resolves the profile, releases original holds before learn guards, then chooses direct audio or UI. |
| `crates/app/src/main.rs:5747–5813` | Keyboard entry handles text, project picker, F10, filter and held keys before `apply`. |
| `crates/app/src/main.rs:1419–2209` | `run` handles folder picker, questions, Chop, copy, swap, file and normal pads in its existing order. |
| `crates/app/src/main.rs:4648–4768` | MIDI guards F10 and questions, then handles Chop, file, copy, swap and normal hits. |
| `crates/app/src/main.rs:2847–2903` | Chop returns an actual pad only when its UI audio hit is accepted; previews and assign targets return none. |
| `crates/app/src/main.rs:818–875` | UI sender checks project readiness and queue acceptance; release can wait in `pending_audio`. |
| `crates/app/src/main.rs:570–666` | Project replacement clears UI holds and advances the project epoch. |
| `crates/app/src/main.rs:5257–5259` | Keyboard edit resolution is disabled by a question; MIDI still publishes the raw edit flag. |
| `crates/app/src/input_tests.rs:70–174,1894–1952` | Recorded dispatch, real offline queues and explicit fences can prove sound before UI handling. |

The 55 existing `input_tests::` tests passed during exploration. Strict main-spec
validation passed 15 items. These are baseline results, not proof of the proposed fix.
The pending-command outcome follows from the source order; no new race test or hardware
reproduction has been run yet. The first implementation test must observe both real
audio queues before UI delivery, not only inject an App event.

The OpenSpec config still describes the pre-85 status. CLI `openspec list` is authoritative
for the newly created change. Current-status documents are updated in the later docs
step, not mixed into this planning-only artifact edit.

## Goals / Non-Goals

**Goals:** one input-state computation, a pure shared pad-route decision and one pad-hit
handler; coherent publication to MIDI; an explicit, separately tested ordering repair.
The refactor and move keep the current behavior. The ordering step changes a race
outcome. The separately approved hold step below repairs mixed keyboard/MIDI release
targets on Chop before the shared handler. Another separately approved step keeps
late direct notifications from starting Chop actions. Keep original releases, local queue
acceptance and project filtering distinct from UI processing acknowledgements.

**Non-Goals:** a new universal command resolver, priority normalization, UI modes or
badges, raw-profile re-resolution after a command, an engine change or new dependencies.
Do not add the roadmap's debug status: it is not needed to test this contract.

## Decisions

### 1. Represent overlapping state instead of imposing the sketch enum

Add `Focus`, `Mode`, `Focus::pack/unpack` and `route_pad` in `crates/input/src/lib.rs`.
`App::focus()` alone constructs the routing snapshot. Publish it through one
`AtomicU32` in `midi::Shared`, replacing all five old atomics. MIDI reads one snapshot
per message and uses that snapshot for screen, edit, bank and pad-route decisions.

Use `Mode` as a small set of named state flags, not an exclusive foreground enum.
Editing can coexist with marking and Chop, and a question can cover a hand state.
Retain screen and bank plus independent edit and routing flags in the packed value.
Text, picker and F12 learner flags can describe existing context but do not acquire
new pad-blocking effects. Keep the current keyboard early guards. This avoids inventing
an `EditChop`, `EditMarking` or other user mode, or discarding the state below an overlay.
No new bitflags dependency is needed. Test every represented flag combination, screen
and bank, boundary banks 0/7, and invalid screen, bank and reserved-bit encodings.
The packed default must mean normal Pads, bank 1, edit closed.

The shared route decision is only:

| Existing routing context | `route_pad` |
| --- | --- |
| F10 screen learn | Ignore new hits, keep input logging and original releases. |
| Question, Chop, file/copy/swap in hand | UI, without direct audio. |
| Normal performance, including edit or sounding marking | Sound directly for MIDI; keyboard sends through its existing UI audio producer. |

Keyboard text, project picker and F10 handlers still take their keys before this pad
route. The folder picker keeps its existing action consumption. F12 learning and
`midi_run` are not silently promoted to the F10 silence rule. The common table does
not make every source's pre-routing boundary identical. Its `Ignore` result cannot
consume a release of an accepted hold.

Alternative rejected: copying the roadmap's `Text > Picker > Ask > Learn > InHand >
Chop > Marking > Edit` enum. It would change priorities and could suppress MIDI or its
edit layer without an approved behavior delta. The roadmap is not a requirement.

### 2. One semantic pad handler, with explicit source and sound disposition

Both keyboard pad actions and MIDI `PadHit` handling call `App::pad_hit`. It keeps
Chop selection and assignment, silent targets, sounding marking, selection, metering,
splash behavior and undo effects. Keep the existing `apply` and `midi_event` wrappers:
merging the wrappers can change history or cut/loop synchronization. A shared helper
must not add a second history step for a MIDI copy or swap.

Pass an explicit source and sound disposition, rather than only `sounded: bool`:
keyboard or UI-deferred/unattempted, direct accepted, and direct rejected. A rejected
direct hit must not become a UI retry. The existing `accepted: false` value alone
cannot distinguish rejection from permission to start a UI sound. Keep original
velocity. Use a result that names the actual accepted pad, if any, for UI hold tracking.

An accepted direct hit is never sent to audio again. Existing stable-route behavior
must remain intact before the ordering repair is added. Direct notifications keep
their captured absolute pad. Only a hit explicitly deferred for pending input chooses
its current bank at UI consumption, after its earlier commands. Store its original
resolved local pad as well; do not re-run the profile against a new screen.

Keep keyboard pressed-pad bookkeeping separate from MIDI UI holds. Do not use a MIDI
hit to overwrite the keyboard's held-key release target. Keep the existing distinction
between an attempted command in the test log and a command accepted by the real queue.

### 2a. Repair mixed Chop release targets before the shared handler

The owner approved this separate behavior repair after the audit at `82f84fa`.
At `82f84fa`, the MIDI entry calls `chop_pad`, which uses `Pads::apply(PadPress)`
and writes `pressed[pad]`, the target used by keyboard releases. A MIDI hit in a new
bank can therefore replace a still-held keyboard key's original target.
Before the repair, recorded-input tests for both accepted and full-queue-rejected
MIDI hits failed with `release 18: [PadRelease { pad: 34 }]`. The reverse press-order
test already passed. Keep these assertions and all existing 84 tests unchanged.

Add an explicit keyboard/MIDI hit source for Chop and a pad press operation that
keeps the existing sound, selected pad, hit flash, held display and marking logic.
Only keyboard presses write `pressed[pad]`. Keep the MIDI candidate-to-accepted-pad
map and local audio queue acceptance unchanged. This small internal pad API change
is part of the approved repair, not the later common `App::pad_hit` refactor.
Do not change same-absolute-pad voice rules, preview or assignment behavior.

Test keyboard then raw note/learned CC80 in another bank, and the reverse press
order. Check both release orders on Chop, after a screen change, with a question
and with F10. Fill the actual callback queue to reject the MIDI hit: it must not
steal the keyboard target or start when space returns. Compare both real audio
queues and the offline waveform with timestamped engine commands. Keep no-clip,
finite, silence, onset and release timing checks. Write `renders/85-mixed-holds.wav`
for owner listening. Commit this repair separately before task 2.2, after full
`task check`; do not mix in sequencing or the shared handler.

### 2b. Keep late direct notifications out of Chop actions

The owner approved this separate repair after the audit at `7272d92`. At that
baseline, dispatch sends direct notifications with `accepted` true or false, but
App's Chop branch ignores that result and calls `chop_pad`. This can replay an
accepted hit, retry a rejected hit, preview a bare slice or change a plan.
Before the repair, both recorded regressions reached an extra UI
`PadPress { pad: 18, velocity: 0.18110237 }` after Chop opened. Keep those empty-UI
queue assertions. The rejection fixture must explicitly clear the filler's engine
sample: offline Engine starts with built-in pad test tones even when UI pads are empty.

Replace the ambiguous boolean in `Event::PadHit` with an explicit three-way sound
disposition: UI-unattempted, direct accepted, direct rejected. Dispatch records it
when it chooses the original route. Mechanically adapt event constructors and
assertions without changing the meaning or strength of old tests. On Chop only
UI-unattempted hits may call `chop_pad`; direct notifications use the existing
normal notification effects on their captured absolute pad, without a Chop
slice/preview/plan action or a new UI audio hold. Preserve question/F10 guards,
direct releases, project filtering, velocity and Chop-entry hush. Other hand
priorities are not normalized by this repair.

Reproduce both failures before the repair, then keep the assertions. Test raw notes
and learned CC80 with low/high velocity in linked listen, bare-slice listen and
assignment, with and without a bank change. Fill the actual MIDI audio queue with
hits on an unloaded pad for rejection; consume those filler notifications before
the tested hit, without processing audio. Do not replace dispatch or its producer.
Use actual keyboard events to open Chop, then consume the delayed real events.
Compare audio with a timestamped oracle; check finite/no-clip, silence, nearest-block
onset and the normal hush. Write `renders/85-late-midi.wav`. Run full `task check`
and commit this repair separately before 2.2; no acknowledgement protocol belongs here.

The owner also requested an agent-operated terminal pass and confirmed listening
to separate audio, not the sampler. Use only the isolated `target/ux` launch from
`AGENTS.md`; verify the unavailable JACK log and absence of a ptsr PipeWire node
before driving it. Record the keyboard screen pass separately from offline audio,
owner listening and physical-controller acceptance. Do not claim that a TUI capture
checks the sound. Run this pass against the release build before owner acceptance.

### 3. Preserve source boundaries and record divergences

The common pad handler does not authorize a new priority policy. Preserve these
source-audited boundaries and test them before replacing the old chains:

- Pad naming and project selection consume keyboard keys; they are absent from the
  current MIDI silence publication. The Files filter consumes text keys but passes
  releases and Ctrl chords. Do not add global MIDI silence or change these releases.
- F10 and F12 learning differ. Only F10 blocks MIDI actions and hits. Message handling
  still feeds the existing learners before their resolved event is handled.
- A question disables keyboard edit resolution but the MIDI resolver receives the
  raw edit state. Retain both via the same snapshot, not one new effective-edit rule.
- Chop pad handling precedes object targeting in both paths. Do not let a held file
  steal a Chop hit merely because the sketch puts `InHand` first.
- If more than one hand field is set, keyboard currently chooses copy, then swap,
  then file; MIDI chooses file, then copy, then swap. `PadTake` clears the file hand
  but does not clear a copy (`main.rs:1933–1941`); taking a file does not clear a copy
  either (`:2099–2108`, `:3161–3186`). Thus a single exclusive hand enum cannot be
  assumed to represent every reachable state. This is a source-only finding, not
  a reproduced or heard defect. Keep the source-specific choice explicit inside the
  shared handler for overlapping hands; do not silently clear state or choose a new
  common winner. A normalized hand policy needs another owner decision and delta.

These exceptions are compatibility constraints, not proof that every current boundary
is desirable. Add characterization tests with each source's current expectation;
paired parity tests still compare the workflows agreed in 84. If a test contradicts
this audit or a main requirement demands a broader repair, stop before changing code.

### 4. Acknowledge processed UI input, not audio acceptance

Use one dispatcher-owned sequence for successfully sent `Event::Action` events and
UI-routed/deferred `PadHit` events, across all ports. Direct-hit notifications, input
log messages, parameters and original releases do not create this barrier. A UI pad
hit can itself open a question or end a hand state, so numbering only actions is not
sufficient. Do not count an unsent event. Keep an event's sequence metadata available
even if project filtering later discards its contents.

`Shared.ack` is the last consumed sequence. After each numbered event, the UI publishes
the resulting complete Focus, then stores that sequence with Release ordering. Do so
also for a no-op, blocked action and a stale-project event: consuming stale work must
retire its barrier without applying it. Do not reset sequence counters in
`apply_session`; that would let old input acknowledge new work. The two counters start
aligned when the dispatcher starts.

For each MIDI message, read acknowledgement with Acquire before loading Focus. If
local last-sent and acknowledged sequences differ, a new hit goes to the UI without a
direct sound, regardless of the old pad route. This also preserves order when a pending
command enters or leaves Chop, a hand state, a question or F10. UI handling then applies
the resulting existing guard rules; a deferred hit under F10 must not become a learn
confirmation or execute a command. Already accepted releases still go first.

Keep the barrier active through every already queued UI hit. Acknowledging the take-file
command alone must not let a later raw hit race ahead of the target hit that will open
its question. After the last such event is consumed and Focus is published, new normal
hits resume the direct path. Compare equality, not `sent > ack`, and test counter wrap.
No timeout, sleep, blocking acknowledgement wait or audio-callback work is added.

Implementation evidence for task 3.1: the original same-update occupied-target
regression observed peak `0.12420408` before UI delivery, where the expected peak was
zero. The test keeps that silence assertion. `UiSender` numbers only successful
sends. `Shared::input()` reads acknowledgement with Acquire before Focus;
`Shared::consumed()` publishes Focus before its Release store. UI retirement follows
the complete `midi_event` wrapper, including its cut and loop synchronization.
`PadSound::UiUnattempted { deferred }` records whether input was explicitly deferred;
its original local position remains in the captured absolute pad modulo 16. Direct
accepted/rejected notifications keep their old absolute pad and cannot become retries.

The coherent-snapshot and Chop-to-FX dispatcher unit fixtures now consume their UI
work with the production acknowledgement method before sending the next stable-route
message. Their original direct, bank, velocity and release assertions stay intact.
The concurrent Focus writer still races every message snapshot. Separate recorded
integration tests leave UI work pending to test ordering, partial consumption, old
Ignore, both retrigger orders, rejection, disconnect and project replacement.

Alternative rejected: forcing every MIDI hit through App. It would break direct
performance. Also reject reading Focus before a new acknowledgement: that can pair a
new ack with an older Focus and reopen the same race. The acquire/release protocol
and store/read order need dedicated tests as well as raw-MIDI integration tests.

This guarantees order at the dispatcher boundary, not physical timestamps across USB
ports. It does not serialize every knob or profile update. Keyboard-origin publication
races and a control changing its profile meaning before resolution are outside this
narrow controller-command fix.

### 5. Extend existing UI hold identities to deferred actual pad hits

Use the existing device/channel/control/epoch identity and UI candidate identifier for
any hit which can start through App, not only a Chop candidate. Retain a stable candidate
for an unreleased control; only an actual successful UI queue push acquires an App hold.
Store every distinct pad actually started in App's identifier-to-absolute-pads map.
Use `id -> Vec<{absolute pad, deferred}>`: one stable candidate can start more than
one bank's pad while earlier commands still wait. Do not overwrite an earlier accepted
pad or add the same pad twice. A later accepted deferred retrigger keeps the deferred
release flag; a rejected retrigger cannot change it. A deferred bank change can make the actual pad differ from
the candidate's old snapshot bank. Release looks up the accepted actual pads, not
the old candidate absolute value. A direct release removes only its overlapping pad
from each non-deferred candidate, leaving other accepted pads for the original UI
release. An accepted deferred hit keeps its own UI release even when an older direct
hold of the same pad releases. The callback can process the older direct release
before App starts the deferred hit. Clearing that newer hold would leave a gate stuck.
The two accepted hits have distinct release obligations; this is not a second release
of the old direct hold. Keep the pre-existing non-deferred Chop merging rule.

Dispatch processes a release or disconnect before hit-mode guards and sends the original
candidate identity even when the hit is still in the UI queue. FIFO UI delivery then
handles hit before release; there is no need for a synchronous reply to MIDI. A silent,
preview-only or rejected candidate gets no App hold. A rejected retrigger must leave an
older accepted hold intact, with no stolen hold for another control.

Keep the existing direct-first release rule for non-deferred Chop records, so one
original accepted hold does not receive both routes' releases. Do not apply that
merging rule to a newly accepted deferred hit: without a callback acknowledgement,
UI cannot know whether the old direct release has already passed. Keep its own UI
release after its accepted UI press, in that same queue. Preserve pending
release retries, ready-at-send checks, epoch filtering and the no-new-project-release
rule. Do not weaken the 84 full-queue and source-identity tests to fit a new structure.
Test both orderings of direct/UI retriggers and multiple accepted pads for one control.

### 6. Separate steps and proof

1. Artifacts only, then review. No implementation is authorized by CLI file existence.
2. Characterize compatibility and add the Focus/table model without changing routes.
3. Replace shared publication, repair mixed Chop holds and late direct notifications
   in separate behavior steps, then unify pad semantics without the sequence repair.
4. Add same-update race regressions, reproduce failure on the real routes, then repair
   ordering and deferred hold bookkeeping together. Red tests are not a separate
   green commit; keep their final assertions when making this step green.
5. Move dispatch methods unchanged into `crates/app/src/dispatch.rs` in a move-only
   commit: `run`, `apply`, `keyboard_event`, `midi_event` and its inner/UI entry,
   `publish_midi`, `focus` and `pad_hit`. Add only module/import and required
   `pub(super)` visibility plumbing. Do not move the terminal main loop or tests.
6. Finish renders and owner documents separately. Each completed task needs a green
   full `task check` and an atomic signed-off commit before the next task is complete.
   Commit execution still requires the owner's git authorization.

Representation-bound low-level tests must adapt from five atomic fields to the packed
value; keep the same screen/bank/edit/learn/hand assertions, with the owner-approved
Chop fixture correction below. Do not remove, skip or weaken behavioral tests from 84.
Keep exact names for all ten ordering delta scenarios, the three added mixed-hold
scenarios and the two late-notification scenarios.

The owner approved the Chop fixture correction before task 2.1 on 2026-11-03.
At `53d317e`, `midi.rs:on_screen` leaves `hand` false even for Chop, and
`play_pads_on_the_fx_screen_across_screen_and_bank_switches` expects a direct accepted
Chop hit. App always publishes Chop as UI-routed. Correct this artificial screen/route
pair: keep all normal-screen direct assertions, check the real UI event for the Chop
case, and prove Chop-to-FX playback through the production driver and actual audio
queues. Keep original bank, velocity and release checks. This is a test-fixture
correction, not a change to live Chop behavior; do not add a test-only route flag.

Use keyboard events, raw notes and learned CC80 through real profile seeding and
production dispatch. Extend Drive only to split publish/drain boundaries and pause
UI consumption deterministically; never replace the production route. Compare direct
and UI queue commands plus state and undo, then process the real offline callback.
Cover the matrix below, not a sleep-based guess that messages shared a UI frame.

| Situation | Expected result | Disposition |
| --- | --- | --- |
| Take file then hit occupied target, no UI between | Silent target and unchanged replacement rules | Repair now |
| Take then target then another hit | First question remains; no old sound or confirmation | Repair now |
| Bank change / back / no-op then hit | Resulting bank and existing mode, once | Repair now |
| Command from A then pad from B | Same dispatcher order, original device release | Repair now |
| Command then hit then release/disconnect before UI | One accepted hit and its release, no stuck gate | Repair now |
| Deferred UI hit then new direct retrigger, and reverse | Original accepted holds, no cross-route duplicate release | Preserve and extend |
| Full queue, same-control retrigger and another control | No new hold on rejection, old hold and pending release retained | Preserve |
| Old commands/hits/releases across project replacement | No new-project effect; barrier retires on consumption | Preserve and extend |
| Busy UI without pending commands or UI hits | Direct sound before UI, original velocity, no duplicate | Preserve |
| Text, pickers, F12 learn, overlapping hands or edit | Existing source-specific boundaries, no priority repair | Preserve |

Create `renders/85-input-modes.wav` through offline Audio: silence during the file target,
one ordinary direct hit before UI delivery, and a deferred gate with its original
release. Check finite values, clipping, silence, one onset and block-relative timing
within one sample. Compare matching velocities across sources; check low/high MIDI
separately. Regenerate 84's WAV and keep its hash
`f795d105ad3f098d03857203e0eaffed73e782dd1d51bcfdfec233f7d35a4da6`.
Do not claim that a deferred hit has the latency of the direct path.

## Interim keyboard screen check at `8a4481d`

The owner confirmed Bluetooth listening. The agent ran the release build in the
isolated `target/ux` environment from AGENTS.md. Both starts logged `JACK unavailable`,
neither logged `JACK: active`, and PipeWire node lists before, during and after the
run contained no ptsr node. Test samples, roots, config, session, logs and captures
stayed under ignored `target/ux`; no absolute fixture path is added to tracked files.
The terminal reported `no audio` and `no key releases` throughout. These results do
not check sound, key-release delivery in the owner's terminal or physical controls.

| Keyboard pass | Observed result |
| --- | --- |
| F1, Enter, L, pad keys, M, U/O | Help opened; pad edit kept its panel when another pad was selected; marking and bank changes showed their existing states. |
| Chop Y, Enter, P, Tab, a pad, Esc | Two slices, slice edit, both assignment orders and the in-order start pad were visible; cancellation returned to LISTEN. |
| Files filter and file targets | `qwerty` stayed text. The same-file target reported that the file was already there. A different file on an occupied pad asked; a pad under the question did not replace it; Esc kept the file in hand. An empty target received that file. |
| Copy and swap, then Ctrl+Z | The saved copy kept source settings and used its expected ` 2` name suffix. Swap exchanged complete pad records. Each undo restored the saved session exactly. |
| Pad name and project picker | Typed pad letters stayed text. Cancellation changed no pad name and created no project. The project picker's first Esc cleared its typed name; the second closed it. |
| F10 and F12 | Screen learn opened and closed without saving a binding. Audio, Input, MIDI and Paths tabs opened. Detected MIDI ports alone are not a controller pass. |
| Save, restart and quit | Saved pad records were retained. Opening the whole-file pad made the expected fresh chop; Ctrl+Z returned to the saved two-slice chop. The final saved session matched the checkpoint exactly; the Bash launch wrapper printed `UX exit: 0`. |

Local evidence: 59 screen captures in `target/ux/85-late/captures/`, the isolated
log and PipeWire snapshots, plus the saved-session comparisons. Initial screen
expectation timeouts remain there too. They were checked against the existing panel,
Esc, same-file, copy-name and fresh-chop rules; no production code was changed.
The first launch wrapper used zsh's read-only `status` variable after quit. That was
a wrapper error, not proof of the program's exit code. A repeat with an explicit Bash
wrapper verified restart and exit code zero. No live MIDI events were injected and no
physical-controller acceptance is claimed. The agent did not listen to the WAV files.

This was an early check of the committed 2.1b state, not acceptance of 85. Tasks 2.2
and 3.1 were pending at that checkpoint. The final pass below follows both steps;
task 5.2 stays open for the owner's remaining keyboard/listening result.

## Final implementation and isolated screen evidence at `1a24909`

Completed local commits include shared handling `c946891`, ordered input `a70f527`,
move-only extraction `999bc68`, the final render test `c8bbbd4` and owner documents
`1a24909`. The focused read-only follow-up review of 3.1 found no remaining blocker
after the early-direct-release repair. It was not a formal memory-model audit or a
physical test. Parent verification reconstructed main.rs from `a70f527` and compared
all nine moved signatures/bodies: identical except required visibility. No terminal
loop, tests, callback, keys, profiles or saved-data format moved or changed in 4.1.

Full checks after render delivery and documents passed formatting, strict clippy,
workspace tests and the existing benchmark exit gates. Application tests: 657; input:
74; TUI: 101. Strict validation: 16 items passed, zero failed. `task build` completed
the optimized release build. Some check runs still reported
`maximum target (50 %): wall EXCEEDED; thread CPU WITHIN`; the unresolved budget issue
and the archived 84a workload/reporting/exit gates are unchanged.

The agent then ran that release twice with JACK-only, buffer 1024, private XDG paths,
`PIPEWIRE_REMOTE=ptsr-none` and `JACK_DEFAULT_SERVER=ptsr-none`. Both starts logged
`JACK unavailable`; neither logged `JACK: active`. Four PipeWire snapshots before,
after startup, after restart and after exit had no ptsr node. Owner config and audio
connections were not changed. The terminal reported `no audio` and `no key releases`.
The test was isolated from the owner's browser/Bluetooth sound; it did not check sound.

| Final keyboard pass | Observed result |
| --- | --- |
| F1, pad edit, another pad, marking plus edit, U/O | Help, selection, retained edit, marks and both banks showed the existing states. |
| Copy to empty pad, swap occupied pads, each undo | Saved TOML proved source fields and expected copy name; complete settings swapped. Each undo restored the entire checkpoint. |
| Name, Files filter and file target | Name opened/cancelled; pad letters typed the filter. Same-file placement was a no-op. A different occupied target asked; another pad left that first question intact. Esc kept the file in hand, then cancelled it. Saved data matched the checkpoint. |
| Whole-file F3, undo and bare-slice edit | F3 made the expected fresh one-slice chop. Undo restored two slices. The bare slice opened BOUNDS edit, not linked-pad TONE edit. |
| P, Tab and Esc | BY HAND and IN ORDER were visible; cancellation returned to LISTEN without committing a plan. |
| F4, F10 and all F12 tabs | FX opened. F10 picked a silent learn target and closed without saving a binding. Audio, Input, MIDI and Paths opened. Settings Esc returned to Pads. |
| Project text, save, restart and quit | Typed `check` stayed project text; first Esc cleared it and second closed the picker. Two starts restored original pad records. Fresh F3 and undo recovered saved cuts; final TOML matched the full checkpoint. Both wrappers printed `UX exit: 0`. |

Evidence stays under ignored `target/ux/85-final/`: 66 successful captures, session
checkpoints, log and PipeWire snapshots. Five earlier expectation timeouts are retained:
uppercase SWAP instead of its lower-case hint, replacement text split across lines,
BOUNDS instead of TONE for a bare slice, unsupported pad edit on FX, and `[Audio]`
instead of uppercase AUDIO. Screens and the existing routing rules explained them;
only fixture expectations changed. No production repair was made during this pass.
After exit no ptsr process or PipeWire node remained. No live MIDI was injected.
Detected X-Session ports are not a hardware pass. Actual kitty releases, owner listening
and physical-controller behavior remain unverified; no owner box was marked passed.

Final listening delivery is `renders/85-input-modes.wav` (99,072 frames, 2.064 s,
SHA-256 `01fafeb65b32d3bceb00210eb40d4be904dc3ea4d7b029068f09eb07bcf3743f`),
plus `renders/85-mixed-holds.wav` and `renders/85-late-midi.wav`. Every main render
segment has silence/onset/release/finite/no-clip checks or a zero-only target check;
normal/deferred sound is sample-identical to timestamped oracles. Matching-velocity
keyboard, note and button waveforms are identical. Low/high MIDI is checked separately.
The regenerated 84 WAV kept its original hash. The exact segment map and separate
owner checks are in section 85 of `docs/LIVE-CHECKS.md`. The owner is asked to check
keyboard behavior and listen to all four named WAV files before acceptance. Tasks 5.2
and 5.3 remain open for that result and explicit archive permission; no push/PR occurred.

## Owner keyboard result and squash merge approval — 2026-11-03

The owner reported that all checks possible with the keyboard passed.
The owner cannot test a physical controller at present. The owner did not report the terminal or separate WAV listening.
Section 85 of `docs/LIVE-CHECKS.md` records the keyboard result.
The late MIDI notification check remains in the controller pass, not in the completed keyboard pass.
Recorded MIDI and waveform tests do not replace hardware checks or owner listening.

The owner approved squash merge of [PR #4](https://github.com/fl64/ptsr/pull/4) into `master` with these checks open.
This approval replaces the earlier request to preserve separate commits in `master`.
The local branch keeps its signed-off commits. The squash commit must include the signoff.
Tasks 5.2 and 5.3 remain open for the remaining listening result and explicit archive permission.
This approval does not authorize archive or close the callback performance issue.

## Owner acceptance and combined MR — 2026-11-03

The owner accepted change 85, then authorized its archive and a local signed-off commit.
This later decision replaces the earlier archive restriction above.
The owner requested one MR for closure of 85 and work on 86 from `86-screen-registry`.
Keep the archive and status update in a separate commit from 86 specifications and code.
PR #4 is merged at `fd6a566`. Do not change `master` directly.

This is overall acceptance, not a separate result for each listening check.
The earlier successful keyboard result remains recorded. Separate WAV listening remains unreported.
No controller is available, so physical-controller behavior remains unverified.
Keep the unreported listening boxes and controller boxes open in `docs/LIVE-CHECKS.md`.
Acceptance does not resolve callback maxima above the 50% target or slow-core CPU spikes.
The closure step changes no sound, keys, profiles, saved data or Rust code.
Push and MR publication still need a separate request.

## Risks / Trade-offs

- Pending work can temporarily defer a pad from another device too. This follows the
  shared UI state and global dispatcher order; normal performance has no such barrier.
- An acknowledgement stored before Focus, or an action-only barrier, reopens a race.
  Check intermediate publication points and queued UI targets explicitly.
- A boolean sound result can turn rejection into a retry. Keep dispositions separate
  and assert commands in both actual queues, not only the attempted-command log.
- A pending bank change can misdirect a release. Key UI holds by original candidate
  identity and accepted absolute pad, and retain epoch/source tests.
- A common handler can change undo or key priority. Preserve wrappers and source
  exceptions, characterize overlap, and stop on a new divergence.
- Benchmark maxima already exceed the 50% target on some runs. Keep the 84a report,
  workload and exit checks unchanged; a green p99 exit is not a maximum-budget pass.

## Migration Plan

No saved-data or config migration is needed. The new state is internal to one running
process. Publish a valid initial Focus before starting MIDI dispatch. At each step,
run full `task check`; run strict OpenSpec validation and release build before owner
acceptance. Do not change scheduler settings, owner configuration or audio routing.

Rollback, if requested, is an atomic revert on the work branch, with git authorization;
there is no user-data conversion to undo. Archive only with explicit owner acceptance
through the OpenSpec CLI, then verify main-spec scenario coverage. Do not sync main
specs by hand and do not mark physical checks passed because recorded tests passed.

## Open questions

- Can an older direct release retire a new deferred retrigger of the same pad?
  Default for 3.1: no. Independent callback progress can consume that old release
  before the deferred press exists. Keep each accepted deferred hit's UI release;
  retain old direct-first merging only for non-deferred Chop hits. The owner requested
  completion without intermediate questions. This clarifies the early-release
  requirement; it changes no engine voice algorithm or audio callback.

- Which physical controller and preset will be available for acceptance? Default:
  leave the hardware pass unverified. Use the measured hardware only in its own later
  change; recorded notes and learned CC tests do not accept the SMC-PAD template.
- Should a later change normalize text/picker/F12-learn behavior and overlapping hand
  priority? Default for 85: no. Preserve and document current source boundaries; do
  not create extra fixes in this refactor. This question does not block the narrow fix.
