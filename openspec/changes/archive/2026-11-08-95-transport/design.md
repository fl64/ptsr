# Design

## Context

See `proposal.md` for scope and owner procedures. The owner requested autonomous pattern development after deferring separate sound-quality work.
The earlier input/profile sequence remains incomplete. This change uses existing keyboard dispatch and MIDI learn, not new holds or a verified SMC-PAD profile.

Verified baseline:

- `crates/engine/src/lib.rs`: commands already enter an engine that can render without hardware. The built-in click is available.
- `crates/engine/src/rt.rs`: command queues retain project epochs and bounded restoration. Stale project commands cannot apply to the current project.
- `crates/input/src/lib.rs`: `transport.play_stop` already means Chop preview. Reusing that name would change saved assignments.
- `crates/app/src/session.rs`: the current format is 3. Formats 1–2 already have pad/chop migration.
- `crates/app/src/view.rs`: title badges support cached formatting and retain pad-mode/safety priority.

The separate roadmap edits remain saved apart from this change. No rejected fade or automatic-group behavior is part of 95.

## Goals / Non-Goals

Goals: an audio-thread clock, exact metronome timing, safe project restoration, keyboard/MIDI routes and visible state.
Keep timing and rendering usable in offline tests. Preserve existing pad, preview and effect processing.

Non-goals: pattern scheduling, count-in, half-time, controller hold detection, new device defaults and sound corrections.
The pattern clock follows project BPM directly. Later pattern work can reuse it without adding a second timer.
The owner deferred separate metronome visuals on the live waveform. Keep its current mixed-output display in 95.
Colored beat marks or separate music/metronome traces remain ideas, not approved requirements.
The owner also deferred diagnosis of a click when Space starts Chop preview of `rec-20261031-194857.wav`.
Moving the start removed the audible symptom in the owner's test. The playback mode and exact source frame remain unverified.
Do not treat the source-file measurements as a reproduction or add automatic boundary shifts or attack fades in 95.

## Decisions

### Clock and metronome

Add `crates/engine/src/transport.rs`. Keep musical position as fractional ticks at 960 ticks per quarter note.
Each processed audio frame advances by `bpm * 960 / (60 * sample_rate)` ticks while playing.
Retain the next beat index independently of metronome enable state. Round beat onsets to the nearest output frame.
A tempo change changes future increments, not position or the next beat index.

Use a dedicated, preallocated metronome playhead reading the existing click sample. Mix it before existing master gain.
This prevents a click from stealing one of the 32 pad voices or needing garbage-queue capacity.
Stopping or muting prevents new clicks. A current click finishes its existing short tail.
A project transition uses the existing transition fade for its old click and resets its clock and preferences.
No new universal fade or sound processing is necessary.

Rejected: a UI timer, because interface delays would move beats. Rejected: ordinary pad-voice allocation for every automatic click, because it can steal pad polyphony.

### Commands and published state

Add `SetTempo`, `TransportPlay`, `TransportStop`, `TransportToggle` and `SetMetronome` engine commands.
Explicit start/stop are idempotent. Toggle runs on the audio thread so consecutive queued toggles cannot use one stale UI snapshot.
Tempo commands reject non-finite values, clamp finite values and round to tenths.

Publish one bounded transport snapshot through atomics after each callback. Use a revision around related fields, as existing pad-position publication does.
A reader makes one attempt rather than waits for the audio thread. It keeps a safe stopped fallback during project restoration.
The UI never advances position itself. Title formatting depends on tempo, play state, metronome and whole beat, not every fractional tick.

Rejected: a shared mutex or owning sample transfer in transport state. Neither is necessary.

### Input and compatibility

Use `transport.toggle` for the new general action. Preserve `transport.play_stop` as Chop preview and `metronome.click` as the existing test click.
Keep global Space overridden by the existing Chop/Files map entries.
Add Ctrl+Up/Down, explicit Ctrl+Shift+Up/Down and Alt+M. No plain or Shift pad-block command changes.
Add new default conflicts to the existing owner-keymap warning mechanism.
Help enumerates effective bindings, not hidden global entries. Keep displayed arrow names separate from keymap parsing.
Retain the existing EDIT-only band. Transport controls remain available in F1.

Add GLOBAL button rows and an absolute `transport.tempo` knob to MIDI learn. Use the existing profile, learned-binding and takeover paths.
Factory X-Session roles and the SMC-PAD example stay unchanged. Physical SMC-PAD PLAY hold and bank-2 knob roles still need changes 89/91.
Do not pretend that recorded MIDI establishes physical acceptance.

Tempo and metronome preferences live in application transport state. Existing project-tagged command queues deliver changes.
Keyboard tempo buttons and MIDI tempo controls update the same state, dirty flag and value popup.
The title observes accepted engine state, not an optimistic running flag.
Combine clock text with the first mode badge. Keep a separate clock badge when no mode is active.
Try compact labels before removing a lower-priority badge. Reserve space for compact mode, question and recording badges before file details.
This keeps transport and safety feedback visible together at 80 columns.
Compact clock text removes spaces, not tempo, position or indicators.
Crowded Chop titles use `#i/n` and `Nsl` before losing variant or slice-count identity.

### Persistence

Format 4 adds `bpm` and `metronome`. New sessions default to 120.0 BPM and metronome off.
Read formats 1–3 using existing migration, then supply missing transport defaults.
Refuse non-finite or out-of-range saved tempo and malformed metronome values before replacing current work.
Never serialize running state or position. Apply project preferences through its new epoch and restore marker.
Save refusal and target validation happen before stopping the source, using the existing project-switch sequence.

An older binary refuses format 4. Rollback needs a compatible project backup or a newer binary. Do not add an automatic downgrade.

## File plan

- Engine: `crates/engine/src/{lib,rt,transport}.rs`, new `crates/engine/tests/transport.rs`, existing `realtime_no_alloc.rs` and callback benchmark if needed for active transport.
- Input: `crates/input/src/{lib,midi}.rs` and `config/keymap.toml`.
- Application: `crates/app/src/{main,dispatch,session,saving,view,values}.rs`, new transport module and recorded-input tests.
- Display: `crates/tui/src/lib.rs` title compaction and regression tests. Retain mode and safety priority.
- Documentation: this change's artifacts and owner procedures. Update current-capability text only after implementation and verification.
- Generated evidence: `renders/95-transport.wav`, screen catalogue where affected and local logs under `target/95-transport/`.

No dependency, device-default or effect change is part of this plan.

## Boundary cases

Automated tests cover the closed rows. These tests do not establish owner listening, terminal or physical-controller acceptance.

| Case | Required result | Status | Verification |
| --- | --- | --- | --- |
| Beat at a block edge, including an empty block | Exactly one onset, unchanged clock on empty input | Closed | Engine transport renders across 1/64/128/1000/4096-frame blocks |
| Tempo changes between beats | Continuous position, no replayed beat | Closed | Engine render: 120 to 90 BPM at an exact block boundary |
| Metronome muted for several beats | Clock advances, no queued catch-up clicks | Closed | Offline audio and position assertions |
| Two toggles arrive before one callback | Both execute in order | Closed | Real command queue test |
| 32 pads sound on a beat | No metronome voice stealing | Closed | Compare all 32 pad tails with metronome on and off |
| Old project commands remain queued | New project stays stopped with its own preferences | Closed | Real epoch queue and App transition with saturation |
| Target read or source save fails | Source clock and preferences remain unchanged | Closed | Project-transition regression for both refusals |
| A newer snapshot interrupts UI reading | No waiting or mixed transport state | Closed | Concurrent snapshot test with stale-epoch rejection |
| Owner Space or tempo binding overrides a default | Owner action stays, one applicable warning appears | Closed | Shipped-map overlay tests and unchanged owner-file bytes |
| Mode, question, clock and long Chop details compete at 80 columns | Required identities remain readable | Closed | Real screen draws with compound placement, 128 slices and chop 128 of 129 |
| Full polyphony and active metronome | Keep existing performance targets and report peaks | Open | p99 exit checks pass. Earlier maximum failures remain unexplained |
| Hardware is unavailable | No claim of physical-controller acceptance | Open | Separate owner procedure after hardware returns |

## Risks / Trade-offs

- Clock drift or rounding at boundaries: compare measured onsets and positions across sample rates and block sizes.
- New display text can hide safety feedback: test both supported terminal sizes and long labels through existing screen generation.
- Added work can affect the callback budget: retain all checks and measure active transport with full polyphony.
- Terminal or desktop shortcuts can consume modified arrows: recorded-event tests check program routing only. Owner terminal verification remains separate.
- The current physical profile is incomplete: MIDI learn provides supported routes without inventing device facts or claiming changes 88–93a are complete.

## Open questions

- Physical-controller and terminal delivery need owner verification. Default: retain the specified keys, existing remapping and learned MIDI routes until evidence requires another choice.
- An earlier callback maximum exceeded the target. Default: retain all limits and report the failure in tasks. Defer performance fixes for a separate owner decision. Do not attribute peaks to transport without a control run.
