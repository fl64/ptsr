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
The refactor and move keep the current behavior. Only the ordering step changes a race
outcome. Keep original releases, local queue acceptance and project filtering distinct
from UI processing acknowledgements.

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
Store the pad actually started in App's identifier-to-absolute-pad map. A deferred bank
change can make that pad differ from the candidate's old snapshot bank. Its release
must look up the accepted actual pad, not assume the old candidate absolute value.

Dispatch processes a release or disconnect before hit-mode guards and sends the original
candidate identity even when the hit is still in the UI queue. FIFO UI delivery then
handles hit before release; there is no need for a synchronous reply to MIDI. A silent,
preview-only or rejected candidate gets no App hold. A rejected retrigger must leave an
older accepted hold intact, with no stolen hold for another control.

Keep the existing direct-first release rule when direct and UI records overlap, so
one original accepted hold does not receive both routes' releases. Preserve pending
release retries, ready-at-send checks, epoch filtering and the no-new-project-release
rule. Do not weaken the 84 full-queue and source-identity tests to fit a new structure.
Test both orderings of direct/UI retriggers and multiple accepted pads for one control.

### 6. Separate steps and proof

1. Artifacts only, then review. No implementation is authorized by CLI file existence.
2. Characterize compatibility and add the Focus/table model without changing routes.
3. Replace shared publication, then unify pad semantics without the sequence repair.
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
value; keep the same screen/bank/edit/learn/hand assertions. Do not remove, skip or
weaken behavioral tests from 84. Keep exact names for all ten delta scenarios.

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

- Before task 2.1: may the low-level Chop-to-FX fixture be corrected to match
  production? At `53d317e`, `midi.rs:on_screen` sets the screen and bank but leaves
  `hand` false, even for Chop. `play_pads_on_the_fx_screen_across_screen_and_bank_switches`
  therefore expects a direct accepted Chop hit. App publication always sets the UI
  route for Chop; `route_pad` deliberately keeps that real rule. The old artificial
  screen/route pair cannot be expressed by the approved complete Focus. Proposed:
  retain all normal-screen direct assertions, correct the Chop case to the real UI
  route and prove its transition to direct FX playback through the production driver,
  keeping velocity, original bank and release checks. Do not add a test-only route
  flag or change live Chop behavior. Default: pause task 2.1 without changing this
  existing test until the owner approves the fixture correction.
- Which physical controller and preset will be available for acceptance? Default:
  leave the hardware pass unverified. Use the measured hardware only in its own later
  change; recorded notes and learned CC tests do not accept the SMC-PAD template.
- Should a later change normalize text/picker/F12-learn behavior and overlapping hand
  priority? Default for 85: no. Preserve and document current source boundaries; do
  not create extra fixes in this refactor. This question does not block the narrow fix.
