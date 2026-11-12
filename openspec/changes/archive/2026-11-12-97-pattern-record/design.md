# Design

## Context

See [the proposal](proposal.md) for the owner goal and acceptance procedure.
This is a proposed implementation design, not implemented behavior or owner acceptance of each detail.

### Verified baseline

- Change 96 provides A1 playback and the checked raw model in `crates/engine/src/pattern.rs`.
- `crates/engine/src/sequencer.rs` owns loop scheduling and note-owned releases.
- `crates/engine/src/rt.rs` accepts project-tagged commands at block boundaries.
- `crates/input/src/lib.rs` publishes one shared routing snapshot. Chop currently routes pad input through the UI.
- `crates/app/src/dispatch.rs` hushes sounding pads whenever Chop opens.
- `crates/app/src/main.rs` provides bounded pad/FX Undo. Its snapshots do not include the raw pattern.
- `crates/app/src/saving.rs` currently saves dirty work without a recording-pass boundary.
- Format 5 and `PatternState` already store bars, quantize, strength and four integers per raw hit.
- `config/keymap.toml` leaves Ctrl+R, Alt+N, Alt+E and Alt+Z available for these defaults.

The separate `repo-recording-controls` branch proposes consistent RECORD tap behavior.
Use its workflow as design input, not as implemented device support.
Do not merge its unrelated documents or the separate usability-rule branch into this change.

## Goals / Non-Goals

**Goals:** Capture what the production pad route accepts. Keep the audio clock authoritative.
Bound callback work and retain data across delayed feedback, retries and project boundaries.

**Non-Goals:** Add no input timestamps, step grid, controller layers, redo or new persistent recording format.
Do not correct existing manual-to-manual release semantics as part of recording.

## Decisions

### One target and one pass

Ctrl+R and `pattern.record` always address A1.
Starting without A1 creates an empty two-bar A1 with the existing default grid and strength.
This empty pattern remains after count-in cancellation. Creation alone does not make a recorded-pass Undo step.

Use Idle, CountIn, Recording and Finishing phases.
Erasure is a flag within Recording, not a second recorder.
Refuse starts during project restoration, unavailable audio or pending pad placement.
Keep the current screen, bank, selection and pad sound settings.

From stopped, CountIn lasts 3840 musical ticks with four metronome beats.
Its clock is separate from the pattern position, but uses the same audio-frame timing and tempo.
The pattern stays stopped until count-in ends. Start its first loop at tick zero once.
Force the count-in click without changing the saved metronome preference.
Tempo changes affect the remaining count-in duration without repeating a beat.

From playback, start at the next accepted command boundary without a restart or count-in.
A second toggle cancels CountIn, or ends Recording while playback continues.
A transport Stop closes the pass before resetting the clock.
Repeated explicit start/end commands are idempotent within a pass identity.
Keyboard repeats and command releases do not toggle.

### Capture at the sound boundary

Add a bounded recording-feedback queue of 512 events from engine to App.
Capture only accepted manual pad commands for the ready project.
Do not capture sequencer output, metronome, previews, placement input or derived UI notifications.
An accepted empty or muted pad may still produce a raw note. It must not create a false PLAY indicator.

The timestamp is the audio block's accepted start tick, not UI time or MIDI receipt time.
This preserves existing live latency of up to one actual audio buffer.
Offline tests can supply exact event offsets. They must not claim better live input accuracy.
Round captured musical ticks to the nearest integer, with ties later.
Store press position modulo loop length. Retain absolute ticks for gate duration and ordering.
Convert accepted gain to MIDI velocity by rounding `gain * 127`, bounded to 1–127.
A zero-velocity release is not a recorded onset.

Each event carries project epoch, pass identity and an increasing capture sequence.
The App must ignore old identities, even if the old project returns later.
Use one coherent published snapshot for epoch, phase, position, count-in beat, capture sequence and drop counters.
The final pass boundary remains observable even when the event queue is full.

### First implementation boundary: capture protocol

Task 1.1 adds the engine protocol before keyboard or App integration.
An explicit start requires running transport, an accepted pattern and a capture queue.
Stopped starts remain inactive until task 2.1 adds count-in.
This intermediate state is not usable pattern recording.

Allocate the 512-event queue before audio processing.
Events contain only copied values: epoch, pass, sequence, accepted absolute tick and onset or release data.
Onsets also retain rounded loop position, absolute pad, velocity and pad mode.
Production commands use offset zero. Offline commands can supply an exact frame offset.

Use separate explicit start, end and completion-acknowledgement commands.
A pass number must be nonzero and greater than earlier accepted pass numbers in that project epoch.
Repeated or mismatched commands do nothing.
The accepted end enters Finishing and freezes its tick and final sequence outside the event queue.
Stop accepts that end before resetting transport.

Completion acknowledgement requires the matching pass, final sequence and an empty feedback queue.
Until that acknowledgement, another start cannot replace the retained end state.
Queue overflow counts omitted events and omitted onsets separately.
It does not postpone an event for later delivery.
A project switch clears the accepted recording state and rejects commands from the previous epoch.
Events already in the queue retain their original identities.

Gate completion, count-in, playback publication and App recovery remain later tasks.
Do not claim their full scenarios from capture-protocol tests alone.

### Gate completion and recovery

One-shot hits can enter playback after capture. Keep their raw gate value at zero.
For a Gate pad, pair the accepted onset with its release and store musical duration.
An unfinished Gate hit stays pending and cannot play as a new sequenced note.
At pass end, close pending gates at the accepted end tick before committing the pass.
Clamp a positive duration to 1–245760 ticks.
A repeated onset on one pad closes its previous pending gate at that onset.
Keep existing manual sound and release behavior when several inputs share that pad.

Reserve bounded storage for pending gates and terminal state outside the feedback queue.
A lost release must not leave an unfinished saved gate or attach to a later press.
On capture overflow, keep existing accepted rows and manual sound. Count omitted onsets and show a warning.
Recover gate completion from the retained terminal state. Never fabricate or replay omitted onsets.
Refuse further raw rows at 4096, including hidden rows and pending Gate reservations.
Erasure can free capacity. Hitting the limit does not stop manual playing or the old pattern.

### Second implementation boundary: Gate capture

Task 1.2 extends the engine protocol. It does not connect recording to App input or saving.
Allocate terminal slots for at most 4096 accepted onsets before audio processing.
Keep the feedback queue at 512 events. A larger queue alone cannot prevent lost releases.
Expose a separate read-only recovery handle. Each bounded read can fail during a concurrent write.
Readers must not wait for a callback.

An accepted Gate onset reserves one raw row and one terminal slot.
Identify the slot by project epoch, pass, onset sequence and slot index.
Retain its accepted absolute start tick. Release, retrigger or pass end determines its duration.
Round the musical interval to the nearest tick, with ties later, then clamp to 1–245760.
One-shot rows have zero duration. Pending Gate rows cannot enter playback.
A dropped or limit-refused retrigger still closes the preceding accepted Gate at its accepted tick.
It does not reserve a row or become a recoverable onset.

Recovery requires the original queued onset and matching terminal identity.
It cannot enumerate omitted onsets or pair a completion with a later press.
Retain completion data until the matching pass acknowledgement permits another pass.
Stop closes known gates before resetting the clock. Project reset invalidates old recovery data.
Manual sound release still follows the existing per-pad rules, not new per-source ownership rules.

Keep the source raw-row count in the compiled pattern.
Count source rows before compilation excludes hidden hits or resolves collisions.
At pass start, reserve that count. Accepted new onsets, including pending Gates, consume remaining rows.
Count queue omissions and limit refusals without changing manual sound or existing playback.
Task 2.3 connects capture draining and pattern publication. Task 5.1 adds erasure and space reuse.
This boundary does not claim those complete owner scenarios.

Generate `renders/97-pattern-capture-gates.wav` from synthetic sound through the production engine.
Compare recorded Gate playback with an independent manual reference.
Check both channels for finite samples, clipping, silence, onset timing and release cuts.
The complete recording demonstration remains task 7.3.

The Gate fixture uses a generated 55 Hz tone at 48 kHz.
Capture starts at tick 3600, tempo changes from 120 to 240 BPM at tick 3840, and release arrives at tick 4080.
The expected duration is 480 ticks. A separate manual command sequence checks completed playback at 120 BPM.
The listening file lasts 6.9375 seconds. Its expected onsets are at frames 90000, 207000 and 303000.
The first hold lasts 0.1875 seconds. The two repeats each hold for 0.25 seconds.
Check both channels against the reference within 0.000001 per sample.
Require repeat onsets within one frame, peak between 0.03 and 0.04, maximum step below 0.004, and final silence.
These numerical checks do not replace owner listening.

### Prevent immediate and delayed duplicates

The plan's `skip_lap` sketch alone is insufficient at a wrapped quantize seam.
A hit near the end can quantize to zero, where its first occurrence in the next lap is legitimate.
A slow UI can also publish a captured hit after its first eligible occurrence already passed.

Give raw rows runtime identities outside the saved four-integer representation.
Retain identity through grid changes, erasure and recompile. Regenerate it after project load.
Retain the accepted capture lap before rounding the onset tick.
Rounding a late onset to the seam must not postpone its first repeat by another complete loop.
Compile a first eligible occurrence strictly after the captured absolute onset.
For a forward quantized tick in the capture lap, defer playback to the following lap.
For a wrapped zero tick, the next lap's zero is eligible.
Pending Gates become eligible only after their duration is complete.

Replacement seeks from the current audio boundary. It never catches up missed occurrences.
Retain bounded per-occurrence playback history through in-pass recompiles.
A changed grid or collision winner cannot play an already consumed occurrence again.
Old hits keep their eligibility when new rows join them.
Stopping and restarting clears runtime eligibility so saved raw hits play normally from zero.

### Preserve the previous rhythm during overdub

The owner approved these playback rules after the sampler-manual review:

- Manual hits sound immediately. Their fresh rows add no same-lap sequenced trigger.
- A late hit that quantizes to zero first repeats at the next eligible loop start.
- An ineligible fresh row does not suppress an eligible old row at the same compiled pad and tick.

Select eligible collision rows before applying the existing winner order from change 96.
That order is higher velocity, longer gate, then earlier raw row.
Once a stronger fresh row becomes eligible, it can win without deleting the older raw row.
While a pass records, keep all raw collision rows.

**Decision change, 2026-11-11.** The earlier decision kept all collision rows after the pass, so that another grid could separate them later.
The owner found the result unclear: one sound plays, but the event strip of change 98 shows two hits, and removal must choose between them.
The owner then approved replacement at pass completion.
When a pass completes, each fresh row replaces older rows of the same pad at the same compiled tick. The most recently captured row stays.
The rules above stay unchanged while the pass records, so the old hit plays until the pass ends.
Pass Undo restores the replaced rows. Loading a project does not remove saved same-tick rows.
Trade-off: a finer grid can no longer separate two hits that a pass replaced.
With quantize Off, hits seldom share a tick, so recording stays additive in practice.
An interrupted pass, for example by quit, ends with the same replacement as a completed pass.
Consequence in the current loop: a fresh row first plays in a later loop. Assume a manual hit before its quantized step, and a pass end before that step.
Then the replaced row does not play at that step in the current loop. The manual hit was the sound of that step, so no double hit follows.
Implementation: the App removes the replaced rows when the pass reaches its end, before the pass acknowledgement.
It publishes the result through the same stamped or unstamped publication as other pass edits. The audio row reservation can stay above the row count.

Manual playing remains independent of the previous rhythm.
A manual hit 20 ms before an old same-pad pattern hit leaves both accepted sounds intact.
This pair is not a duplicate trigger from the fresh row.
Suppressing the old hit would change overdub into partial replacement, so this design rejects that option.

The manual review supports additive recording, not a universal collision policy.
The [Maschine manual](https://docs.native-instruments.com/pdf-guides/maschine_277/MASCHINE_2.0_MK3_Manual_English_2_7_7.pdf) separates input quantize modes and describes additive overdub on pages 80 and 461–462.
The [SP-404MKII manual](https://static.roland.com/assets/media/pdf/SP-404MK2_v5_reference_eng03_W.pdf) describes loop recording without pattern replacement on pages 59–60.
Neither cited section establishes the first-repeat or collision rules above. Those rules are the owner's approved ptsr behavior.

Task 1.3 must test eligible collision selection and independent manual sound through the production engine.
Compare both output channels against independent manual references.
Include forward quantize, wrapped zero, delayed publication, grid changes, pending Gates and Stop followed by restart.
Generate `renders/97-pattern-repeat.wav` for later owner listening.
This approval does not establish implementation, a passed test or listening acceptance.

### Third implementation boundary: recorded-row playback

Task 1.3 adds an engine-only compilation path. Keep ordinary `RawPattern::compile()` compatible.
The new `compile_recorded` method accepts one runtime descriptor per raw row.
Each descriptor has a checked slot and generation, an optional capture origin, and a pending-Gate flag.
At most 4096 descriptors can exist. Active descriptors must use different slots.
These descriptors never enter format 5 or change the saved raw order.

Capture events retain the transport run identity. A new stopped-to-playing transition starts a new run.
Derive the capture lap from the accepted absolute tick before rounding, not the saved modulo tick.
Use the same nearest-audio-frame loop boundary as sequenced playback.
This keeps the first seam frame in the correct lap despite floating-point clock accumulation.
A fresh row first becomes eligible after its capture lap in that run.
After Stop and restart, old origins no longer restrict completed rows. Pending Gates remain silent.

Recorded compilation retains collision candidates in the existing winner order.
The audio cursor selects an eligible candidate and consumes that occurrence for all colliding row identities.
A preallocated 4096-slot history retains the latest consumed lap for each identity through replacement and grid changes.
A reused slot with a different generation cannot inherit the earlier row's history.
Seeking a replacement also consumes elapsed occurrences, including rows first published after their scheduled time.
This prevents a later grid change from reviving a missed occurrence.

Ordinary replacement still releases old sequenced notes. It must not clear the recorded occurrence history.
Stop, restart and project reset clear that history. Keep retirement backpressure and note-owned releases unchanged.
Before an in-pass grid edit, App publication must assign identities to existing rows under the previous grid.
Every later in-pass compilation must retain these descriptors. App publication itself remains task 2.3.
Erasure, input controls and saving also remain later tasks.

Boundary review for task 1.3:

| Case | Expected result | Status | Evidence |
| --- | --- | --- | --- |
| Forward quantize | No new same-lap trigger | Closed for the engine boundary | Four callback-size sequences and both-channel references |
| Before, on and after the seam | First repeat at the first eligible zero | Closed for the engine boundary | One-frame neighbours at 40, 120, 172 and 300 BPM |
| Late publication then grid edit | Missed occurrences stay missed | Closed for the engine boundary | Three-loop delay, repeated replacements and reference sound |
| Stronger fresh collision | Old eligible hit stays audible | Closed for the engine boundary | Immediate manual hit, old hit 20 ms later, then the new winner |
| Collision split or winner change | No consumed row repeats | Closed for the engine boundary | Raw rows survive recompile, grid separation and slot generation changes |
| Pending Gate then completion | No unfinished or revived occurrence | Closed for the engine boundary | Completion before and after the scheduled tick |
| Stop or project reset | No old eligibility restriction or stale update | Closed for the engine boundary | Stop, toggle, restart and project-epoch tests |
| Full candidate set and retirement queue | No callback allocation or deallocation | Closed for the engine boundary | 4096 candidates and saturated retirement |
| App delivery and active-recording budget | Complete recording and measured callback cost | Open | Tasks 2.3 and 8.1 |

These checks do not establish hardware input, owner listening or complete recording.

### Fourth implementation boundary: audio-timed count in

The owner approved the nine-file task-2.1 plan and `renders/97-pattern-count-in.wav`.
This boundary connects the existing engine commands, not keyboard, MIDI, App publication or screen controls.
Keep the capture queue, accepted pattern and fresh nonzero pass prerequisites.
A stopped start now enters CountIn. A playing start retains immediate capture without restarting transport.
The earlier task-1.1 stopped-start refusal describes only that historical implementation boundary.

Use a separate audio clock for the 3840-tick count-in and force only that clock's metronome.
Keep the saved transport metronome setting unchanged, including explicit preference changes during count-in.
Publish count-in ticks and the remaining beat number, from 4 to 1, through the existing coherent recording snapshot.
Zero-length callbacks accept commands but cannot advance either clock or consume a click.
Split the audio span at the nearest-frame end boundary before scheduling A1's first note.
Start one new transport run and capture at zero. Do not delay that transition until the next complete buffer.
Do not force a fifth click. After the boundary, ordinary metronome preference controls the click.

An explicit end for the accepted pass cancels CountIn immediately to Idle without Finishing or capture events.
Retain the consumed pass identity so late repeated starts cannot reopen it.
Stop also cancels and leaves transport stopped. Preserve warm-up voices and their ordinary per-pad releases.
Play or a stopped-to-playing transport toggle cancels preparation and starts ordinary playback without recording.
This preserves ordinary transport control. It does not record before the fourth beat ends.
Repeated starts and mismatched ends cannot reset or shorten the count-in.
A project switch discards the old timer with the old recording state. Old queued commands cannot start it again.

Reuse the existing metronome mixer so automatic clicks cannot steal any of the 32 pad voices.
Do not use an interface timer or allocate a new sound voice for each count-in beat.
Keep ordinary prepared-pattern playback, manual release rules, format 5 and dependencies unchanged.
Pattern-setting refusal remains an App control responsibility in task 4.1, not a new backend pattern-replacement policy here.

Boundary review after engine tests:

| Crossing | Expected result | Status | Evidence |
| --- | --- | --- | --- |
| Buffer end, zero-length callback and zero tick | Four clicks, one exact start, no missed first A1 hit | Closed for the engine boundary | 1, 128, 1024 and mixed blocks at 44.1, 48 and 96 kHz, plus starts inside a callback |
| Tempo change between beats | Continuous musical position, no doubled or missing beat | Closed for the engine boundary | Independent click and pattern reference with a mid-beat tempo change |
| Warm-up Gate across start or cancellation | No captured warm-up onset, unchanged manual sound and release | Closed for the engine boundary | Both-channel manual reference, retained release and new run identities |
| Metronome preference off, on or changed | Temporary forced clicks only, then the current preference | Closed for the engine boundary | Exact click amplitudes, timing and post-start silence |
| End, Stop, Play, toggle and repeats | Cancel or ordinary playback, never early capture or stale restart | Closed for the engine boundary | Production command queues, all remaining beat values and frame-boundary cancellation |
| Project switch with stale commands | No old countdown or pass in the new project | Closed for the engine boundary | Project-tagged queues, readiness and saturated retirement |
| Full retirement queue and 32 pad voices | No stealing, callback allocation, deallocation or wait | Closed for the tested engine paths | Allocator counters and independent full-polyphony mix. Active-recording performance remains unmeasured. |
| Keyboard, MIDI, Undo, controls and audio loss | Complete owner workflow | Open, outside this boundary | Tasks 2.2, 2.3, 3, 4, 5 and 7 |

Generate the listening file from built-in clicks and generated tones at 48 kHz.
Use three sections: normal count-in, count-in with a tempo change, and cancellation with a warm-up Gate held.
Check both channels against independent ordinary metronome and manual-pad references within 0.000001 per sample.
Require finite samples, peak below 1, maximum adjacent step below 0.11 and silent gaps after each section.
Check click and pattern onsets within one output frame.
The file lasts 10 seconds, with 480000 stereo float32 frames at 48 kHz.
Both channel peaks are 0.5. Their maximum adjacent step is 0.096080772.
The last 4800 frames of each section are silent.

Listening procedure for `renders/97-pattern-count-in.wav`:

1. Listen from 0 to 3 seconds. Check four clicks, then two short tones at 2 and 2.5 seconds.
2. Listen from 3 to 7 seconds. The tempo changes from 120 to 90 BPM at 3.25 seconds.
3. Check four clicks, then tones at 5.583333 and 6.25 seconds in that second section.
4. Listen from 7 to 10 seconds. Check two clicks and a low warm-up tone from 7.75 seconds.
5. Check cancellation at 7.875 seconds. The low tone must continue until release at 8.125 seconds, then fade for 100 ms.
6. Check that the cancelled section has no third click or pattern start. Check all attacks and releases for unwanted clicks.

Owner listening remains unverified. These engine tests do not establish keyboard, controller or complete recording acceptance.

### Fifth implementation boundary: interrupted capture recovery

The owner approved the five-file task-2.2 plan and `renders/97-pattern-interrupted.wav`.
The existing Stop and explicit end already complete Gate slots before the clock resets.
The remaining engine case is a finish request that cannot receive another audio callback.
`Recovery::completed` currently returns no row for a pending Gate. Preserve that live-playback rule.

Add `Recovery::interrupted(onset, boundary)` for use only after audio becomes unavailable.
The boundary must be the last coherent recording snapshot, not a UI timer or an estimate of lost audio time.
The future App caller must detect audio loss and retain this snapshot before starting a new audio instance.
This method returns a recoverable raw hit. It does not change engine phase, send an acknowledgement or resume sound.
The App will finish its own pass from these rows in tasks 2.3 and 6.1.
Do not claim that this engine-only boundary already makes saving, exit or keyboard recording work.

Require the original queued onset and matching project, pass, sequence range and loop length.
Accept only Recording or Finishing boundaries with finite non-negative time and an accepted onset at or before that boundary.
Reject Idle, CountIn, wrong identities, invalid values, unaccepted slot indices and events beyond the retained final sequence.
Keep one-shot gate length zero. A pending Gate closes at the retained boundary.
Round its musical interval with ties later and clamp it to 1–245760 ticks, as for normal completion.
Keep an earlier known Gate completion. Never extend it to the interruption time.
If completion lies after the supplied coherent boundary, cap the result at that boundary.

Distinguish a stable pending slot from a stale or concurrently changing slot.
A stale or changing read returns no row instead of a fabricated completion.
Use bounded atomic reads with identity checks before and after recovery. Never wait for another callback or a writer.
Reuse the preallocated terminal slots. Do not add a queue, dependency, saved field or audio-thread write for recovery.
Repeated reads of the same retained boundary return the same raw row, not extra captured events.
The App must update that original row identity, not append another copy on a retry.
A new pass or project reset invalidates the earlier recovery context.
A new audio instance must use new queues and recovery storage. App pass identities must remain fresh within the project epoch.

Boundary cases after engine implementation:

| Crossing | Expected result | Status | Evidence |
| --- | --- | --- | --- |
| Finish request queued, callback absent | Retain accepted rows and close pending Gates at the last accepted tick | Closed in engine tests | Production queues, retained Stats snapshot, then destruction of the isolated engine |
| Completed Gate and later pending Gate | Keep the earlier duration and close only the pending note | Closed in engine tests | Released and retriggered notes, pending Gate across a seam and tempo change |
| Stop then restart | Use the pre-reset boundary, never resume an old pass | Closed in engine tests | Matching completion, repeated reads, acknowledgement and new pass |
| Full feedback and omitted release | Recover only accepted onsets, not omitted hits | Closed in engine tests | Full 512-event queue, terminal slots and omission counters |
| Wrong phase, epoch, pass or sequence | No row from an unrelated or unaccepted context | Closed in engine tests | Invalid boundary matrix and new project/pass tests |
| Pending slot versus changing slot | A valid pending Gate can close, but an unstable read cannot invent a row | Closed in engine tests | Odd revisions, changed slot identities and concurrent terminal writes |
| Minimum, midpoint and maximum interval | Existing rounding and duration limits | Closed in engine tests | Zero interval, midpoint neighbours and 245760-tick boundary |
| Allocation and deallocation | Bounded recovery and unchanged callback memory behavior | Closed in allocator test | All 4096 accepted slots, repeated recovery, end, Stop and restart |
| Audio-loss detection, Undo, save and exit | Complete owner recovery workflow | Open, outside this boundary | App integration in tasks 2.3, 5.3 and 6.1 |

The implementation adds one read-only recovery method. Its bounded reads distinguish stable pending duration zero from an inconsistent terminal slot.
The live `completed` method still excludes pending Gates. No new audio-thread write or recovery queue exists.
New coverage includes ten production-queue integration/render tests, two atomic unit tests and one allocator test.
These tests cover this engine boundary, not the complete saveable-take workflow.

The listening example uses permitted synthetic tones and recovered rows through ordinary production pattern playback.
Both channels match independent manual commands within 0.000001 per sample.
The file has 216000 stereo float32 frames at 48 kHz and lasts 4.5 seconds.
Channel peaks are 0.031496063 and 0.027559055. Maximum adjacent steps are 0.000227078 and 0.000297725.
All samples are finite. Attacks occur within one frame of 0, 1 and 3 seconds.
The first Gate lasts 240 ticks, or 125 ms. The next two Gates last 480 ticks, or 250 ms each.
Each note has a 10 ms release. The last 4800 frames are silent.

Listening procedure for `renders/97-pattern-interrupted.wav`:

1. Listen from 0 to 1 second. Check one short low tone at the start.
2. Listen from 1 to 4.5 seconds. Check equal longer tones at 1 and 3 seconds.
3. Check that the first note stays shorter. Recovery must not extend its known duration.
4. Check that the recovered note repeats with the same duration. Check attacks and releases for unwanted clicks.

Owner listening remains separate from these automatic checks.

### Compact controls before the step editor

`pattern.controls` uses Alt+N and opens a compact panel on the current screen.
Its three rows are Bars, Quantize and Strength.
I/K or arrows select a row. J/L or arrows change its value.
Enter, Esc or Alt+N closes the panel without changing screens or stopping playback.
Pad keys remain playable. The panel has no editable hit grid or new F-key screen.

Bars uses 1–64 in steps of one, clamped at both ends.
Allow length changes only while Idle. Keep hidden raw rows under the existing 96 rules.
Quantize walks Off, 1/4, 1/8, 1/16, 1/32, 1/64, 1/8T, 1/16T and 1/32T without wrapping.
Strength steps by 10 from 0–100 without wrapping.
Allow grid and strength changes during Recording. They do not change raw timing.
CountIn and Finishing refuse pattern-setting edits with a phase-specific explanation.

Learned `pattern.bars`, `pattern.quantize` and `pattern.strength` accept absolute knobs.
Map 0–127 into the permitted ordered values by nearest index, with ties later.
Use existing jump, pickup and scale response separately per device and control.
Expose `pattern.param_prev`, `pattern.param_next`, `pattern.value_dec` and `pattern.value_inc` as buttons for devices without knobs.
A value action requires the pattern panel. Direct named parameter knobs do not.

### Pitch Gate feedback states its A1-only scope

The owner could not hear a Pitch Gate difference during manual play.
Change 96 deliberately excludes manual hits from the cut. Keep that sound contract unchanged.
The change-96 panel shows the saved `PG on` or `PG off` value and dims ineligible source or pitch states.
`crates/app/src/main.rs` derives eligibility from a linked slice and positive pitch.
`crates/tui/src/lib.rs` uses that value for color, without explaining manual-play exclusion in the mode row.
The owner approved a clearer existing indicator, text reasons and visible manual-play help in change 97.
This is an interface clarification, not a new sound effect.
The partial implementation and its checks appear in [the task record](tasks.md#pitch-gate-feedback-work-in-progress).

Separate the saved switch from applicability. Neither value proves that a sounding note has a cut.
Use the selected pad's current settings, not transport activity, a pad flash or a manual press.
When On, test source first, then pitch. A whole file, detached piece or empty pad needs a linked slice.
A linked slice at zero or negative pitch needs `Pitch > 0`.
A selection, source or pitch change refreshes the explanation without changing the saved switch.

Proposed compact labels and visible help:

| Saved switch and applicability | Mode-row label | Visible help text |
| --- | --- | --- |
| Off, with any source or pitch | `PG off` | `PG off, manual unchanged` |
| On, linked slice and positive pitch | `PG A1` | `PG on: A1 only, manual unchanged` |
| On, without a linked slice | `PG n/a` | `PG on: needs linked slice, manual unchanged` |
| On, linked slice and non-positive pitch | `PG n/a` | `PG on: needs Pitch > 0, manual unchanged` |

The labels are at most six cells, no wider than the existing `PG off` label.
The help texts describe one setting, not several instructions.
Keep the state in the existing mode row and its explanation in the existing three help rows.
Do not add a title badge, move the frame or create a new pad-input mode.
Color may reinforce the meaning, but must not be the only evidence of a restriction.
Keep safety priorities and access to existing controls. F1 retains the full effective-key explanation.

Toggle popups retain the target pad identity, saved On or Off value and current applicability.
F1 explains the positive-pitch linked-slice rule, next-grid cut, unchanged manual hits and unchanged previews.
Keep existing Alt+G and learned-button routes. Add no binding, audio message or persistent field.

An eligible setting still says A1-only without A1 or with stopped transport.
Mixed manual and A1 playback does not make a manual hit subject to Pitch Gate.
Do not replace the text with a global active or inactive claim during mixed playback.
This step does not add per-note cut telemetry or make short slices longer.
Tests must cover both sizes, edit states, hidden pads, source and pitch changes, selection and mixed playback.
Add named tests for the new scenarios instead of relying on change 96's earlier state-and-color test.
Tasks 7.1 and 7.2 own this implementation. The earlier specification edit added no UI code or new WAV.

The owner approved the nine-file implementation plan for this feedback boundary.
Use the first two existing help rows for compact commands on Pads.
Use the third existing help row for the selected pad's Pitch Gate explanation from the table above.
F1 retains commands that no longer fit the compact rows. This reduces visible command hints, not available controls.
Other screens keep their three command rows. Learning keeps priority over ordinary Pads help.
Keep notice and safety rows unchanged. Do not infer applicability from the source label.
Replace the drawing snapshot's saved-switch and eligibility fields with one four-state Pitch Gate value.
Derive it from the saved switch, source and pitch on each view update.
The mode row, popup and F1 use that same value. No new cached application state is necessary.
The existing popup reserves at most 26 cells for the value. Keep that geometry and the pad identity.
Use `PG off`, `PG on: A1 only`, `PG on: needs linked slice` or `PG on: needs Pitch > 0` there.
F1 includes the compact mode label and this explicit setting description.
A stale popup retains the existing invalidation rules. Current panel and help explanations follow the current selection.

Boundary cases for this step:

| Situation | Expected result | Status before code | Check |
| --- | --- | --- | --- |
| On with an empty, whole-file or detached source and non-positive pitch | Source restriction takes precedence | Contract closed, test open | Production source states at negative, zero and positive pitch |
| Selection, source or pitch changes after a toggle | Refresh panel and help without changing the saved switch | Contract closed, test open | Two pads, linked and detached source transitions, pitch across zero |
| Manual hit with absent, stopped or playing A1 | Keep A1-only scope, not a sounding-cut claim | Contract closed, test open | Keyboard and raw MIDI with offline audio |
| Narrow frame with edit, hidden pads or safety warnings | Keep mode row, explicit help and safety positions | Contract closed, test open | 80x26 and 120x40 buffer assertions |
| Effective-key override, learned note or CC button | Retain the actual control and popup target | Preserve existing routing, test open | Production dispatch, F1 and popup assertions |
| Learning or a modal question overlaps feedback | Keep existing learning, question and popup priorities | Preserve existing priority, test open | Offline frame and existing overlay tests |

At review, automated tests cover these six boundaries. Live terminal, physical-controller and listening checks remain open.
The popup test also covers the 26-cell restriction and unchanged pad identity.

Do not change engine behavior, saved formats, bindings or dependencies in this boundary.
Active-recording indicators and the complete change-97 screen matrix remain later work.

### Erasure without controller hold recognition

Use Alt+E and `pattern.erase` as a latched toggle during Recording.
A held pad is an erasure target, not a new sounding or recorded note.
An ordinary pad held before erasure starts must first release before becoming an erasure target.
Releasing an erasure target never creates a hit.
Turning erasure off clears its held-target set. Playing again requires a fresh pad press.

Erase raw rows whose compiled onset falls within a held target's accepted playhead interval.
Use half-open intervals `[press, release)`. A same-frame release excludes that frame.
A held target spanning a seam covers both sides. A whole lap covers all audible occurrences of that pad.
Erase every raw row behind a matching collision, not just its winning row.
Preserve other pads, rows outside the traversed interval and hidden rows outside the current length.
Apply erasure before a matching sequenced onset, so that onset does not sound during erasure.

The engine tracks held targets at audio time. The App erases rows using acknowledged intervals.
Do not infer an interval from two UI redraws. A frozen UI must not erase unrelated time after release.
Retain erased row identities until the compiled pattern acknowledges their removal, so later loops cannot revive them.
Bound pending intervals and report overflow. Never erase outside the acknowledged interval.
When the terminal has no releases, one pad press targets erasure for one loop, then clears itself.
This fallback does not change one-shot playback outside erasure.

Reject held Backspace as the default: it conflicts with existing removal and needs release support.
The latch works with current learned buttons and does not depend on future controller holds.

### Undo without changing unrelated work

A pass snapshot contains the pattern raw rows and settings before Recording starts.
During a pass, combine recorded rows, erased rows and grid/strength changes into that single snapshot.
At completion, append one isolated pattern Undo step. Do not merge consecutive passes by elapsed time.
No pattern difference means no pass step.
Restore only pattern state. Keep pad parameters, shared effects, sample files and Chop edits.

Ctrl+Z on Pads uses normal common history, including the completed pass.
During an active pass, it first completes and undoes that pass instead of undoing an older edit.
An empty active pass does not fall through to an older common edit.
Alt+Z and learned `pattern.undo` provide this pass action on every screen.
When Idle, this action requires a pattern pass at the top of common history.
Refuse skipping newer common edits, especially a pad swap. Explain that the newer edit must be undone first.
Chop's ordinary Ctrl+Z retains its local cut/plan role outside an active pass.
Redo remains outside this change. A new pass does not depend on redo support.

### Shared input and Chop

Pattern controls and recording temporarily make Chop pads address real absolute pads.
Draw those real pads, including empty ones, rather than misleading slice-keyboard cells.
While CountIn runs, pads can sound as practice but produce no raw recording events.
Ending recording restores Chop listen routing for new presses.
An earlier held sound retains its original release address across this change, bank changes and screen changes.

Do not hush pad voices when entering Chop during CountIn, Recording or Finishing.
Otherwise, retain the existing Chop-entry behavior. Do not resolve 96's separate solo-preview question here.
Space on Chop and Files keeps its preview role, including during recording.
Previews never enter the pattern. A learned global transport control still addresses transport.

Text, questions, pickers and learn keep their established source-specific priorities.
Do not make all MIDI pads silent merely because keyboard text input is open.
Capture only commands that the real route accepts. Selecting a command for learn never executes it.
New start/end/erase commands must precede following raw MIDI pad input through production dispatch.

Refuse starting with file/copy/swap placement, marking or Chop assignment active.
During CountIn, Recording and Finishing, refuse entry into those modes before they can consume pads.
Keep volume, pitch, FX, screen changes and ordinary preview available.
Higher-priority overlays close before Esc can cancel count-in or finish recording.
An unclaimed Esc during Recording finishes the pass without discarding it. Undo can then discard that pass.

### Saving and project lifetime

Suspend autosave from CountIn through Finishing.
Manual save ends the pass, drains feedback through its accepted boundary, then invokes normal saving.
After Recording, it leaves transport running. Cancelling CountIn leaves transport stopped.
Report `Session saved` only after actual write success.
A failed write keeps the completed pass, Undo and pending save state.
Quit completes the same boundary before invoking the existing exit-save policy.
Neither operation waits inside the audio callback or blocks input processing while completion is pending.

Refuse project picker entry during these phases with a finish-recording instruction.
After completion, existing source-save, same-folder and target-validation rules apply unchanged.
No project transition writes partial Gate rows or carries recording state into the target.
Restart always restores Idle and stopped transport, never count-in, erasure or a pending pass.

If audio becomes unavailable, end at its last coherent accepted tick.
Retain the recoverable raw take, close known gates and report the interruption and capture losses.
Do not remain permanently in Finishing or block saving and exit because no callback can acknowledge completion.
The App can then save under existing read/write protection.

### Sixth implementation boundary: App capture publication

Task 2.3 connects the accepted capture queue to raw A1 rows and compiled playback.
The owner approved ten files and one local WAV, including the engine publication extension below.
It does not enable recording keys, learned recording controls, erasure, Undo, saving or project-format changes.
The owner still cannot start recording from the interface after this boundary.

Each audio instance owns a new capture consumer and its engine recovery storage.
Production JACK, cpal and offline tests use the same setup.
Before an internal start request, initialize existing row identities under the current grid.
Publish that compiled pattern before the start command through the existing project queue.
Keep these runtime descriptors separate from the four-integer saved rows.
Retain descriptors through later publication, grid compilation and completed-pass pad swaps.
No slot reuse occurs in this boundary. Erasure must add generation changes before it reuses a slot.

Read one coherent accepted snapshot before draining events.
Consume at most the fixed queue capacity per poll.
Leave events beyond that snapshot sequence for a later poll.
Discard events from another epoch or pass without changing A1.
Append accepted onsets in their original order. Never reconstruct an omitted onset from counters.
Reserve a raw row for each pending Gate and keep its playback descriptor pending.
Complete that same row from its original onset identity, not a new row on each retry.
Compile all updates with the retained descriptors before publication.
Use the existing pending command queue and retirement backpressure without a second input route.

A Finishing snapshot permits acknowledgement only after all accepted events and Gates reach A1.
Queue the final pattern before its acknowledgement. Repeated polls must not duplicate either transaction.
Count-in cancellation creates no rows. A later pass retains all completed rows in their original order.
Project restoration clears the collector and runtime descriptors before the new epoch can accept input.

After the audio stopped flag, freeze the last coherent accepted snapshot.
Use interrupted recovery to close pending Gates at that snapshot, without a timer or fabricated acknowledgement.
Keep the recovered take in App memory and report interruption and omitted input.
Do not resume collection on that audio instance if its stopped flag changes again.
Complete save, exit, Undo and audio replacement remain separate lifecycle work.

The existing contract forbids placement during CountIn, Recording and Finishing.
This boundary tests descriptor retention when pads swap after a completed pass.
It does not permit swaps during capture or implement the placement-entry guards from task 3.
Those guards must exist before recording controls become available.

| Case | Required result | Boundary status | Check |
| --- | --- | --- | --- |
| UI delay beyond the first eligible occurrence | No fresh double hit or catch-up burst | Closed by focused tests | Keyboard, raw MIDI and learned buttons, both channels, missed first repeat |
| Pending Gate, repeated polls and end | One reserved row, then one completed row | Closed by focused tests | Held note across a lap, one raw row and one accepted acknowledgement |
| Publication while an old Gate sounds | Preserve its accepted release, including Pitch Gate | Closed by focused tests | Initial App publication, later updates, engine control render and full retirement |
| Overflow and empty pads | Keep accepted rows only, with loss warning | Closed by focused tests | 512 retained onsets, 128 omitted onsets and a lost raw MIDI release |
| Restore with old queued events | No old rows in the new project | Closed by focused tests | Actual restoration, fresh epoch and reused pass number |
| Audio stops before end callback | Retain the take at the last accepted tick | Closed by focused tests | Callback removal, repeated recovery and refused reuse of the stopped instance |
| Completed pass followed by swap | Hits and sounds move together, identities remain | Closed by focused tests | Production swap on all three input routes, unchanged descriptors and later reference sound |
| Save, Undo, active placement and recording controls | Full owner workflow | Open, outside this boundary | Later tasks 3, 5 and 6 |

Generate `renders/97-pattern-app-capture.wav` with synthetic sound through the production App and engine queues.
Include a manual hit, later repeats and an overdub over existing rhythm.
Compare both channels with an independent manual-input reference within 0.000001 per sample.
Check finite samples, clipping, attack timing and required silence.

#### Publication blocker and approved scope extension

The ordinary `SetPattern` command calls `replace_pattern(pattern, true)` in `crates/engine/src/lib.rs`.
That path calls `end_pattern_notes` in `crates/engine/src/sequencer.rs`.
It releases sounding Gate notes and cuts sounding Pitch Gate notes before their original deadlines.
Ordinary one-shot tails do not use that release path.
This behavior serves ordinary replacement. Reusing it for each captured-row update would interrupt the previous rhythm.

A controlled offline render replaces A1 with the same compiled pattern after 128 frames.
Both runs use a 960-tick Gate, 120 BPM and a constant synthetic sample at 48 kHz.
At frame 1024, both control channels remain at 0.02. Both replacement channels are zero.
The source and literal output stay under `target/97-app-capture/replacement_repro.rs` and `replacement-repro.log`.
This experiment detects a publication constraint, not a new live-device failure.

The owner approved a separate command for recorded-row publication: `PublishRecordedPattern`.
It must preserve existing sequenced voices, their original release deadlines and recorded occurrence history.
Keep ordinary replacement unchanged. Include the existing retirement backpressure and stale-project rejection rules.
Test Gate and Pitch Gate tails, obsolete commands, full retirement and callback allocation counters.

The approved extension adds three files to the original seven-file scope:
`crates/engine/src/lib.rs`, `crates/engine/src/rt.rs` and `crates/engine/tests/realtime_no_alloc.rs`.
The App tests cover sounding notes through initial identity publication and later capture updates.
Reject changing all ordinary replacements because that would change unrelated playback and project behavior.
The owner authorized this extension before implementation.
The earlier unconnected App draft and failing capture test stay under `target/97-app-capture/draft/`.

The new command carries a compiled pattern without changing the saved project format.
Initial row identities and all capture updates use this command through ready, current-project queues.
Ordinary `SetPattern` retains its existing release behavior.
A full retirement queue delays the complete publication. Discarded publication also retires its pattern outside the callback.
New snapshots cannot acknowledge a pass before its accepted rows reach the publication queue.
After detected audio loss, retain raw rows without queueing publication or acknowledgement to the stopped instance.

#### App boundary verification and listening

The focused App suite includes fourteen tests through production project and input queues.
The render test crosses keyboard, raw MIDI and learned buttons with timely or delayed publication and a completed-pass swap.
Its delayed case checks that no row reaches A1 before the UI resumes after the first eligible occurrence.
The reference keeps the existing per-pad retrigger fade. It does not introduce separate voices for manual and sequenced sources.
The 4.125-second existing pattern onset can therefore fade the earlier manual tail under the unchanged retrigger rule.
This boundary preserves scheduled Gate and Pitch Gate releases during publication, not during a new same-pad hit.

Other tests cover muted pads, MIDI velocities 1/63/127, refused unready input and full command queues.
They check the 4095/4096 raw-row boundary, including hidden rows, and Stop before clock reset.
The engine control render preserves old Gate and Pitch Gate deadlines when the replacement changes grid and duration.
Allocator tests include 4096 descriptors, 32 sounding voices, full retirement and rejected publication during project restoration.
No callback allocation or deallocation occurs in those measured paths.

Collection does not connect session dirty tracking, autosave, manual save, exit or Undo.
Those lifecycle guards must exist before recording controls become available.
All implementation boxes remain open until the required checks and an authorized commit meet the repository rules.

The local WAV is `renders/97-pattern-app-capture.wav`: seven seconds, 48 kHz, stereo float32.
The App render matches an independent manual-input engine reference within 0.000001 per sample on both channels.
A separate WAV reader compares the complete signal with analytic tones and the existing retrigger fade.
Its maximum difference is below 0.000001 on both channels.
It also checks finite samples, clipping, isolated attacks within one frame, the reference click bound and silence.

Listening procedure:

1. Open `renders/97-pattern-app-capture.wav` in a player.
2. Listen for the first low tone at 0.105 seconds and the higher rhythm tone at 0.25 seconds.
3. Listen for the low repeat at 2.125 seconds and the higher repeat at 2.25 seconds.
4. Listen for two nearby low hits at 4.105 and 4.125 seconds.
   The first is manual input. The second belongs to the previous rhythm, not a doubled fresh row.
5. Listen for one low hit at 6.125 seconds and the higher tone at 6.25 seconds.
6. Check the final silence and listen for unwanted clicks.

Owner listening remains unverified. This WAV does not establish usable interface recording or live-device acceptance.

### Seventh implementation boundary: completed-pass Undo

The owner approved six files and `renders/97-pattern-pass-undo.wav` for the completed-pass part of task 5.3.
The files are App `main.rs`, `pads.rs`, `pattern_record.rs`, `input_tests/patterns.rs`, and this change's `design.md` and `tasks.md`.
Recording controls remain disabled. Active-pass Undo, global pattern Undo, erasure and save/exit guards remain later tasks.
This boundary does not change the engine, saved format, bindings or dependencies.

Capture the prepared A1 raw rows, settings and runtime descriptors before queueing an internal recording start.
A newly created default A1 therefore remains empty after Undo of its first pass.
Associate that snapshot with the requested project epoch and pass, not the next UI redraw.
Reject another internal start while this request waits for audio acceptance.
Count-in cancellation and refused starts add no history step.

After the accepted finish, drain feedback and complete pending Gates before adding a common-history step.
Emit at most one step for that pass. An interrupted pass uses only its frozen accepted boundary and recoverable rows.
Compare raw pattern state, not transient descriptors. An empty pass or settings changed back to their original values adds no step.
Append each changed pass independently of elapsed time. Keep the existing 100-step common-history bound.
Collect accepted completion before appending a later ordinary edit or executing ordinary Undo.
This preserves the order of a pass and a later pad edit even when completion feedback arrives between UI actions.

Ordinary common Undo restores only the saved A1 snapshot and its descriptors.
Compile the restored state before changing the model or consuming history.
Use `PublishRecordedPattern` so already sounding notes keep their accepted release deadlines.
Only future sequenced occurrences change. Keep pads, FX, Chop state, selection, bank and transport unchanged.
Do not restore an entire session or add a separate out-of-order history stack.
A newer common edit, including a pad swap, remains above the pass and must be undone first through ordinary Undo.
Dedicated cross-screen refusal and active-pass completion remain task 5.4.

Each new row uses its accepted nonzero pass number as its runtime identity generation.
An Undo can remove rows, but cannot rewind accepted pass numbers.
A later pass can therefore reuse a row slot without inheriting the removed row's playback history.
Keep the same identity when a pending Gate completes. Project reset clears both history and capture ownership.
This handles between-pass slot reuse only. In-pass erasure needs its own generation and retirement rules in task 5.1.

This boundary does not mark ongoing capture for autosave or implement recording-aware manual saving.
Ordinary Undo retains its existing unsaved-state behavior. Complete persistence guards must precede enabling recording controls.
Do not claim full erasure/grid-control scenarios from direct model fixtures or internal start commands.

The focused checks close the following cases within this boundary, not the complete recording workflow.

| Crossing | Required result | Focused status | Check |
| --- | --- | --- | --- |
| Two passes within one second | Two independent steps, undo latest only | Closed by tests | Production keyboard, raw notes and learned buttons. The previous timestamp stays inside the join window. |
| Pad, FX and Chop edits during a pass | Pattern Undo preserves those edits | Closed by tests | Compare all pad snapshots, shared FX and saved Chop state. Check note tails separately. |
| Count-in cancellation, empty pass and row-limit refusal | No pass step | Closed by tests | Raw comparison, reverted settings and 4096 hidden rows |
| Held Gate, Stop and repeated completion polls | Complete duration, one step | Closed by tests | Accepted pre-reset tick and final publication before acknowledgement |
| Audio loss before finish acceptance | One recoverable step without a fabricated acknowledgement | Closed by tests | Destroy the isolated callback and retry collection. Undo sends no command to the stopped instance. |
| Completion followed by an ordinary edit | Newer edit remains above the pass | Closed by tests | Delayed collection before history append and before compiling a pad swap |
| Undo, another pass and reused row slots | Fresh identity, no replay from removed rows | Closed by tests | Descriptor comparison within one lap and both-channel manual reference across later laps |
| New project or saturated command queue | No leaked step or lost final publication | Closed by tests | Production restoration, 100-step history bound and queue backpressure |

Generate the local WAV with synthetic tones through the production App and engine queues.
Include the original rhythm, a recorded addition, its later repeat and Undo followed by the original rhythm alone.
Compare both channels with an independent manual-input reference within 0.000001 per sample.
Check finite samples, clipping, level, onset timing within one frame, the reference click bound and final silence.
Owner listening and physical-controller checks remain unverified.

### Completed-pass verification and listening

The 13 new focused tests cover isolated pass history, unrelated edits, delayed completion and current note tails.
An invalid snapshot test checks compilation before model changes.
The render matrix crosses keyboard, raw MIDI notes and learned buttons with Undo during or after a sounding one-shot.
Gate and Pitch Gate results also match control runs without Undo.
Another render checks a fresh pass after Undo without same-lap replay or a missing later repeat.
These tests use internal recording starts. They do not establish recording-control or complete save/exit behavior.

The local WAV has 432000 frames, 9 seconds, 48000 Hz, stereo float32.
Both channels match the independent manual-input reference within 0.000001 per sample.
A separate analytic check covers reference levels, finite samples, clipping, attacks within one frame, the click bound and final silence.
The 55 Hz source's first nonzero sample is below 0.000000001 because its envelope starts slowly.
The attack check uses a 0.000000000001 threshold. The waveform tolerance remains 0.000001.

Listening procedure:

1. Open `renders/97-pattern-pass-undo.wav`.
2. Listen for the original higher tone at 0.25 seconds.
3. Listen for the manual lower tone at 2.105 seconds and the original higher tone at 2.25 seconds.
4. Listen for the lower repeat at 4.125 seconds and the higher tone at 4.25 seconds.
5. After Undo at 4.375 seconds, listen for only the higher tone at 6.25 and 8.25 seconds.
6. Check the final silence and listen for unwanted clicks.

Owner listening remains unverified. Recording controls remain disabled.

### Eighth implementation boundary: capture-aware autosave

The owner approved six files for unsaved-state tracking and autosave during pattern work.
The files are App `main.rs`, `saving.rs`, `pattern_record.rs`, `input_tests/patterns.rs`, and this change's `design.md` and `tasks.md`.
This is part of task 6.1, not complete manual-save or exit handling.
Recording controls remain disabled. The engine, bindings, format 5 and dependencies stay unchanged.
Sound stays unchanged, so this boundary adds no WAV.

Mark collected raw rows and Gate completion as unsaved work through the existing save state.
Creating default A1 also marks unsaved work, including when count-in later cancels.
Do not mark unchanged existing A1 merely because runtime descriptors initialize or an empty pass finishes.
A changed completed pass also marks unsaved work when it contains only pattern-setting changes.
Keep the original pending-save timestamp instead of postponing it for every captured event.

Before autosave decides whether to write, collect current accepted capture through the existing UI-thread path.
Defer autosave while a start awaits acceptance, during CountIn or Recording, and throughout Finishing until accepted Idle.
Completing model rows alone does not bypass a live Finishing phase with an acknowledgement still queued.
A new pass before the next save suspends autosave again without losing the pending-save state.
No partial Gate row or runtime recording state can enter an automatic write.

After audio loss, first collect recoverable rows and complete known Gates at the frozen accepted boundary.
Permit normal autosave once collection is complete, without requiring acknowledgement from the stopped instance.
A lost count-in or a request with no accepted boundary contributes no captured rows and does not block autosave indefinitely.
Keep autosave deferred while recoverable pending rows or compilation remain incomplete.
Project reset discards old capture ownership and its autosave deferral.

Apply this deferral only in the autosave path.
The existing eligibility helper also controls exit saving, which remains outside this boundary.
Preserve read protection, the pause after write failure, missing-folder refusal and the silent successful-autosave policy.
A failed write keeps the take, its common Undo step and pending-save status.
Do not add retries that bypass the existing successful manual-retry requirement.

The table records focused results, not acceptance of the complete change.
The tests use production App collection, stored-file reads and the existing input routes.

| Crossing | Required result | Focused status | Check |
| --- | --- | --- | --- |
| Requested start and delayed acceptance | No write before the accepted phase is known | Closed | Saturated 1024-command queue, repeated start refusal and unchanged stored bytes |
| Count-in cancellation and new default A1 | Empty A1 remains unsaved, then saves without a pass Undo step | Closed | Project and scratch serialization, canceled and interrupted count-in |
| Accepted onsets, pending Gate and elapsed save timer | Pending indicator stays visible, stored file stays unchanged | Closed | Keyboard, raw MIDI notes and learned pad buttons; exact final duration 240 ticks |
| Finish accepted, acknowledgement queued | Save waits through live Finishing | Closed | Saturated command queue; complete model does not permit an early write |
| Audio loss before finish acceptance | Save only recovered rows with known Gate duration | Closed | Recording and Finishing loss, no fabricated acknowledgement, saved data and Undo |
| Empty pass or runtime-only descriptors | No new unsaved-state timestamp | Closed | Existing raw A1 comparison and a setting-only changed pass |
| Undo, another pass and project reset | Correct pending state, no old-phase deferral | Closed | Original timestamp survives; Undo and later project save use the correct rows |
| Read protection, write refusal and missing folder | Existing refusal and retry rules remain | Closed | Damaged/newer format, occupied temporary file, missing project folder and manual retry after Idle |
| Audio loss without any accepted snapshot | No invented rows or permanent start deferral | Closed | Collector unit test with an unaccepted start and stopped audio |

Manual save during recording, quit finalization, project-picker guards, erasure and active/global Undo remain later work.
The known shared temporary-directory failure in keymap tests remains a separate repair.
Do not enable recording controls or describe the required check as green on this boundary's authority.

#### Capture-aware autosave verification

All 13 focused autosave tests passed. The preceding 14 App-capture tests and 13 completed-pass Undo tests also passed.
The two existing save-feedback and refusal tests passed through their keyboard and MIDI routes.
The required `task check` passed all 879 App tests, then failed its audit with 56 uncovered scenarios.
The known keymap race did not fail this run, but its cause remains unchanged.
All 29 implementation boxes remain open. No commit or publication occurred.

Separate engine and TUI suites passed. The release build passed in the main working folder.
The six existing change-97 WAV files remain byte-identical. This boundary adds no WAV.
Details, commands and literal results are in `target/97-autosave/final-results.log`.
Ordinary playback benchmarks passed this run, but do not measure active capture, publication or autosave.
Historical callback-budget failures remain unresolved. Physical-controller acceptance remains unverified.

### Ninth implementation boundary: manual save completion

The owner approved eight files for manual save during pattern work, then approved `crates/app/src/notices.rs` as the ninth file.
The original files are App `main.rs`, `saving.rs`, `dispatch.rs`, `pattern_record.rs`, `input_tests/patterns.rs`, and this change's projects delta, design and tasks.
This boundary implements the manual-save portion of task 6.1, not exit or picker guards.
Recording bindings remain disabled. The audio engine, format 5 and dependencies stay unchanged.

Ctrl+S and learned note/CC save controls use the same App action.
Collect current accepted capture before deciding whether a manual write can start.
If a start awaits acceptance, queue the matching end after that start.
Cancel CountIn or finish Recording through the existing command queue.
If Finishing already runs, wait for its completion instead of requesting another end.
Keep one pending manual request with its original project, epoch and input source.
Repeated save input joins that request without another end command or write.
A new recording start waits until the pending save attempt completes.
Normal pad input, releases and screen navigation continue while the request waits.

Display `Finishing A1 before saving.` while an active boundary delays the request.
Wait for accepted Idle on live audio, including final publication and acknowledgement under queue pressure.
If audio stops, first complete recoverable capture at its frozen accepted boundary, without a fabricated acknowledgement.
Then invoke the existing writer once. A successful write uses the existing five-visible-second `Session saved` notice.
Keep the original request source for success and refusal notices, even when another input acts during completion.
Do not write early because an autosave timer expires. Autosave must not replace or duplicate the manual attempt.
The existing read protection and missing-project-folder policy remain unchanged.
On refusal, retain completed rows, the pass Undo step and pending-save status.
Do not retry automatically. A later explicit save can retry under the existing protection rules.

Project restoration clears the old request before replacing state or publishing a new epoch.
Check project ownership again before a deferred write. A delayed request cannot write or announce success in another project.
Full project-picker entry guards and audio-instance replacement remain later work.
The existing stopped flag, rather than dropping the App audio handle, models audio loss in these tests.
Manual save without active pattern work keeps its existing immediate behavior.
This boundary needs no blocking wait, sleep loop or new worker thread.

| Crossing | Required result | Status after tests | Check |
| --- | --- | --- | --- |
| Save and pending Gate feedback | Complete accepted duration before any success | Closed | Keyboard, raw MIDI notes and learned CC; project and scratch bytes |
| Save and CountIn or unaccepted start | Cancel the same request, keep default A1, leave transport stopped | Closed | Delayed acceptance and count-in cancellation |
| Repeated save and full queue | One end, one write, one Undo step and original input source | Closed | Production queues, later pads/releases and screen navigation |
| Manual save and elapsed autosave timer | Only the manual attempt owns write and feedback | Closed | Stored bytes, notice history and refusal without retry |
| Audio stops before finish or acknowledgement | Recover known Gates and save without waiting for another callback | Closed | Stopped flag with frozen Recording or Finishing |
| Read protection or write refusal after finish | Keep completed rows, history and pending state | Closed | Damaged/newer files, occupied temporary file and missing folder |
| Project replacement before deferred completion | No old-request write or success in the target | Closed | App restoration and explicit current-project retry |
| Save in Idle with no audio or existing refusal | Preserve established feedback and retry behavior | Closed | Existing saving and input suites |
| Progress behind Warn/Error or after replacement | Cancel only the owned entry without stale feedback | Closed | Production routes and notice identity/deadline/eviction tests |

This boundary adds no WAV because it reuses existing end and publication behavior without an engine change.
Re-run capture, completed-pass Undo and autosave tests, the required check and the main-folder release build.
Do not close implementation boxes or enable recording controls while required checks fail.

#### Manual-save notice blocker

The first new test reproduced success before capture completion. The implementation now passes that test across all three input routes.
Before the approved extension, nine focused tests compiled. Eight passed and one failed at the expiry of the success notice.
The failed test returned `Finishing A1 before saving.` again after `Session saved` expired.
The ordinary notice queue retains an interrupted Info notice when a higher-priority Done notice starts.
The focused failure is in `pattern_manual_save_repeats_join_and_preserve_origin_while_pads_and_navigation_continue`.
Its literal output is in `target/97-manual-save/focused-second.log`.

The original eight-file boundary had no identity-based removal of the progress notice.
Do not clear the whole queue or advance a fictitious clock, because other notices must keep their state.
The ninth file, `crates/app/src/notices.rs`, exposes cancellation of one owned transient notice through its existing identity mechanism.
The pending save releases only its own progress notice on completion or cancellation, and retains notice history.
Tests check cancellation behind a warning and preserve unrelated errors.
The owner approved this extension before the notice-queue edit.
Keep the original notice insertion API for existing callers. The owned insertion path returns the entry's existing identity token.
The pending manual save keeps that token. Completion, refusal and project cancellation remove only its matching entry.
An already expired or replaced entry makes cancellation a no-op, even when a new notice has identical text and origin.
Cancellation does not acknowledge an unrelated error, remove history, change visibility or advance an artificial clock.

After the extension, all 13 focused tests pass across keyboard, learned note and learned CC input.
The tests cover active and hidden progress, Warn/Error priority, equal text with distinct identities, stale or expired tokens and queue eviction.
Project reset removes hidden progress without changing the other warning. Success expiry and refusal acknowledgement cannot revive progress.
The notice history remains intact under its existing limit. Cancellation keeps unrelated deadlines and error identities.
The tests close these notice cases for this boundary. Full workflow, required-check and performance limits remain separate.

### Tenth implementation boundary: quit and project guards

The owner approved continuation through all existing change-97 plans without further per-boundary confirmation.
This boundary uses App `main.rs`, `saving.rs`, `dispatch.rs`, `pattern_record.rs`, `input_tests/patterns.rs`, plus projects delta, design and tasks.
The ninth file is engine `capture.rs`, with only a recovery documentation clarification for a closed prefix during final shutdown.
Its existing atomic reads already reject stale or concurrent data. No engine executable logic or format changes.
Complete quit and picker safety before enabling recording controls.

On quit, freeze the collector at the latest coherent current-project snapshot, not at UI time.
Close the accepted prefix locally through the existing interrupted-Gate recovery rules, without classifying a normal quit as audio loss.
Do not change the backend stopped flag, send a fabricated acknowledgement or require another callback.
If a live snapshot or terminal read overlaps its writer, retain the pending quit and retry on ordinary UI turns. Do not spin or sleep.
An absent live snapshot is not proof that a requested start remains unaccepted. A lost unaccepted start with no snapshot can still close.
The normal main loop exits only when the accepted prefix is complete. A new recording start refuses while quit is pending.
Project restore discards an old pending quit. Earlier explicit manual save retains its source and one write attempt.
Only then apply existing exit eligibility, read protection, missing-folder refusal and failure-paused retry rules.
A fatal terminal I/O error or process crash does not gain a crash-recovery guarantee.

Picker entry, direct open and creation refuse active capture, a requested start or pending quit before any source save or target side effect.
After completion, preserve same-folder handling, target validation and source-save order.
Do not add project I/O to a callback or new queue/format fields.
Reject mandatory End/ack waiting because an unavailable backend cannot complete it.

| Crossing | Required result | Status after review | Evidence |
| --- | --- | --- | --- |
| Quit and pending Gate, without another callback | Complete accepted duration and one pass Undo before writing | Closed for this boundary | Keyboard quit after keyboard/raw MIDI holds |
| Quit and unaccepted start or CountIn | Keep default A1, no warm-up rows or fabricated phase | Closed for this boundary | Scratch/project stored rows |
| Quit and Finishing or full queues | No wait for end/ack and no duplicate rows | Closed for this boundary | Saturated command and capture queues |
| Quit and a prior manual request | One manual result and unchanged source | Closed for this boundary | Production save/quit, success and refusal |
| Quit and concurrent snapshot or terminal reads | UI retries without a placeholder or lost start | Closed for this boundary | Concurrent Gate tests and deterministic unavailable snapshot |
| Picker and all active phases or requested start | Unchanged take, clock and filesystem with a recovery notice | Closed for this boundary | Keyboard and learned raw MIDI |
| Idle picker and existing file protection | Preserve prior transition and retry behavior | Closed for this boundary | Production transition/save suites |

These tests close only this local boundary. The required scenario audit, recording activation and live acceptance remain incomplete.

This boundary adds no WAV. Existing sound references must remain byte-identical.
Physical controllers, owner projects, commits and publication remain outside this execution permission.

### Eleventh implementation boundary: accepted engine erasure

This boundary adds engine erasure before input activation or App row removal.
Its ten files are engine `capture.rs`, `lib.rs`, `pattern.rs`, `sequencer.rs`, `rt.rs`, allocation tests, new `pattern_erase.rs`, patterns delta, design and tasks.
It adds `renders/97-pattern-erase.wav`. It does not add dependencies, a file format or physical-device work.

`PatternErase` turns erasure on and off for the matching pass. Other commands start one-loop targets, consume intervals and reclaim Gate slots.
Ordinary accepted pad presses become silent targets only during accepted erasure. Earlier ordinary holds must first release.
Normal pad sound, retrigger and release rules stay unchanged outside erasure. A silent target does not need a voice-retirement slot.
Accept erasure only during Recording with recorded row identities. Repeated enable requests preserve existing holds.
End, Stop, disabling erasure and project reset clear target holds without creating a note.
Use musical time for the one-loop fallback. Exclude the release or expiry endpoint.
Use the sequencer's nearest-frame boundary for automatic expiry. Retain the current ticks per frame in the coherent snapshot.
Keep that field zero outside Recording and Finishing. Freeze it at pass end so later tempo changes cannot change terminal recovery.
The first expiry test observed tick 4079.9999999969605 instead of 4080. A strict floating-point comparison retained the target one frame too long.
Do not loosen the expected duration or audio comparison. Match the existing output-frame scheduling rule.
Process matching due onsets before expiring a one-loop target. An onset just before expiry can share its output frame.
The review test at 44100 Hz and 172 BPM first removes 3839 of 3840 rows after a full loop. It must remove all 3840.
Only silent manual targets bypass voice retirement. Other sequenced pads must retain the ordinary capacity check, including all 16 fading slots.
The full-retirement review test first reproduces a callback panic. Share the capacity check between ordinary voice commands and sequenced playback, not the erasure exception.
The release review test also finds a compiled onset just before release that shares the release frame. Preserve that accepted interval until due scheduling passes it.
Retain at most 4096 scheduling windows in preallocated storage, separate from UI-consumed feedback. Refuse a new target if either store is full.
Commit closed windows against compiled rows before disabling erasure or freezing End/Stop. Preserve raw collision candidates and their accepted birth times.
A row born after an earlier closed interval must survive. Runtime origins therefore retain accepted ticks, transport run, pass and sequence. Format 5 stays unchanged.
Only an accepted close completes an interval. A known one-loop deadline alone does not prove that audio processed its expiry.
An accepted automatic expiry keeps its exact endpoint within the nearest-frame bound. Active progress and a later unaccepted close still clamp to the snapshot.
Freeze the last matching row before terminal capture, even if expiry rounds to the earlier output frame. A later capture in that same frame must survive by pass and sequence order.
Automatic expiry retains the exact musical endpoint, not the nearest frame's floating-point tick. A new run, project reset or accepted pass acknowledgement clears scheduling windows.
After erasure, a preserving publication must carry its accepted pass and sequence. Reject an older erasure prefix without replacing the current pattern.
Retire the rejected reference through the ordinary garbage queue. Do not let stale UI rows return after closed scheduling windows expire.
A publication also applies still-live accepted windows through the current audio tick. The App integration must publish again when the erasure sequence advances, even if its raw rows do not change.

Allocate 4096 interval slots and 4096 removed-row entries before processing audio.
Each interval retains epoch, pass, capture sequence, absolute start, optional terminal end, loop length, grid and strength.
Read active progress only through a coherent accepted boundary. Clamp a later terminal end to a closed prefix.
Keep intervals until explicit UI consumption. A stale consumption command cannot free a reused slot.
Refuse a new target when interval storage is full. Count the omission and keep it silent, without erasing an untracked interval.
A grid or strength publication closes held intervals at that accepted tick and opens their continuation under the new settings.
The App integration will apply intervals to raw rows, including accepted rows not yet published to the sequencer.
It must respect each row's accepted birth time. It must not infer interval duration from redraws.

Before a matching compiled onset, mark every collision candidate removed and suppress its sound.
Retain removed identities across delayed publications and loop turns. Other pads and hidden rows stay intact.
Remove the retained identity only when a later pattern publication no longer contains it.
Do not acknowledge pass completion while any removed identity awaits this publication.
The recovery reader also retains each removal's sequence. A closed prefix cannot acquire a later deletion.

Erasure permits more than 4096 accepted onsets over one pass while at most 4096 raw rows remain live.
The original cumulative Gate-slot index therefore needs reusable storage, not a larger allocation.
Use a fixed free-slot pool and match reclamation by pass, slot and onset sequence.
Reclaim only after UI completion or raw-row removal. Detach a reclaimed pending capture without changing its manual voice.
An omitted onset still has no recoverable row. An exhausted terminal pool counts an omission instead of overwriting retained data.
Keep cumulative onset counters separate from reusable terminal slots.
Counter exhaustion must count an omission, not wrap or silently lose a live reservation. Manual sound remains available.
A pass-owned reservation confirmation retains its snapshot sequence, cumulative onsets, cumulative removals and current raw-row count.
Recompute live reservations as accepted rows plus later accepted onsets minus later removals.
Reject stale or inconsistent confirmations. Never replace live reservations with a delayed UI row count alone.
The App integration must also allocate fresh row generations when it reuses removed row slots.

Reject UI-only suppression because a delayed UI would let matching old hits sound or return on later loops.
Reject one queue for interval starts and releases because a lost release could extend erasure.
Reject an unlimited interval list and a permanently increasing terminal index because callback memory must stay bounded.

| Crossing | Expected result | Status after review | Test boundary |
| --- | --- | --- | --- |
| Erase press and exact release or seam | Half-open interval, other pads and hidden rows unchanged | Closed for this boundary | Engine audio and retained intervals |
| Delayed publication and collisions | All matching identities remain silent across loops | Closed for this boundary | Stale-prefix rejection, live-progress application and independent audio reference |
| Ordinary hold before erasure | Original release, no converted target or synthetic note | Closed for this boundary | Accepted pad commands |
| No-release target and tempo change | Exactly one musical loop | Closed for this boundary | Boundary and tempo matrix |
| Full interval or feedback storage | Bounded refusal, no untracked erasure or callback allocation | Closed for this boundary | Saturated queues and allocator checks |
| Erasure and reused Gate storage | Fresh press cannot inherit or change older completion | Closed for this boundary | More than 4096 captured onsets in one pass |
| Grid publication and active target | Accepted split, no retroactive grid change | Closed for this boundary | Retained settings and old-window exclusion from new compiled positions |
| End, Stop, project reset or stale command | Clear targets and retain only the matching accepted prefix | Closed for this boundary | Frozen-prefix reads, early-expiry terminal stability and project-tagged commands |
| Expiry and later capture in one output frame | Preserve the newly accepted onset, erase the earlier row | Closed for this boundary | Accepted birth ticks, pass and sequence tests |
| Erasure and full voice retirement | Other sequenced pads keep the ordinary capacity rule | Closed for this boundary | 32 sounding plus 16 fading voices and allocation counters |

The listening file uses a generated kick and high tone at 120 BPM for six seconds.
Erasure suppresses the kick at half a second. Its later loop repeats also stay absent. Other quarter-note kicks and high tones remain.
Compare both output channels against an independently compiled reference without that kick, within 0.000001 per sample.
Test finite samples, clipping, levels, timing, cuts and final silence. Owner listening remains separate.
Full keyboard/MIDI activation, App removal, pass Undo and the complete erasure scenarios remain later integration work.

### Twelfth implementation boundary: App erasure and reusable rows

Apply accepted erasure to the App model before recording input activation.
The six files are App `pattern_record.rs`, App pattern input tests, engine `pattern.rs`, patterns delta, design and tasks.
Add `renders/97-pattern-app-erasure.wav`. Preserve format 5, earlier renders and ordinary sound routing.
Do not add recording bindings, the controls panel, dedicated active Undo or new indications in this boundary.

Read matching removal identities before appending accepted onsets. This frees raw space without losing already accepted feedback.
Use reusable row slots with monotonically increasing per-slot generations. Undo must not rewind these generations within a project.
Track pending Gates by row identity, not by mutable vector index.
Reclaim copied Gate completions and those for erased rows, with their original pass and onset sequence.
Queue reclamation after successful compilation. Audio loss and quit recover locally without commands to the closed backend.

Read retained intervals only through the accepted snapshot. Apply each interval's own loop length, grid and strength through the production compiler.
Expose compiled playback descriptors in event order, including collision candidates but excluding hidden rows.
Do not reproduce quantize arithmetic in the App. Preserve raw order when erasing matching rows.
Check accepted birth ticks and closing sequence before erasing a captured row. A later onset in the same output frame must survive.
Keep interval feedback until all accepted onsets in its snapshot prefix reach the model. An unstable read retries on a later UI turn.
Count processed complete intervals, so sparse reused slots cannot cause premature completion.

Publish the changed model before interval consumption, Gate reclamation, reservation confirmation and the final pass acknowledgement.
After erasure, stamp the publication with its accepted pass and sequence. Republish when the erasure sequence advances without a raw change.
Send row reservation confirmation only from a fully collected prefix. Preserve later audio reservations through the engine's counter correction.
Keep compile work pending after failure. Do not emit another completed-pass step or reapply old erasure after that pass already entered Undo history.
Retain the existing save, quit and project guards until collection completes. Complete pass Undo restores rows and settings, not unrelated pad or effect edits.

Reject vector indices as row identities because erasure shifts later rows and can assign an old Gate to a new onset.
Reject immediate interval consumption because uncollected accepted onsets may still need that interval.
Reject a second quantize implementation because it can disagree with production playback at a seam or collision.

| Crossing | Expected result | Current status | Evidence or remaining check |
| --- | --- | --- | --- |
| Removal and a full raw model | Free known removed slots before accepted append | Tested | 4096 raw rows, collisions, a hidden row and delayed accepted append |
| Slot reuse and pending Gate | New generation and original completion ownership | Tested | 4097 accepted onsets, stale consumption, shifted Gate and fresh generations after Undo |
| Closed interval and later same-frame onset | Preserve the new row by accepted order | Tested input paths | Keyboard, raw notes and learned CC pads, including an early rounded expiry |
| Interval and grid change | Use retained settings, retain hidden and untraversed rows | Tested | Nine grids, strengths 0, 50 and 100, three input sources and an unpublished onset |
| Missing or unstable feedback | Retry without waiting or premature acknowledgement | Partly tested | Prefix counts and compile retry exist. An isolated concurrent App erasure-reader test remains open. |
| Completed pass and queued Undo | No old erasure can delete restored rows | Tested | Full 1024-command queue, repeated UI polls, one history step and a later pass |
| Erasure and save, quit or audio loss | Save only a complete owned prefix | Tested | Disposable scratch and project files, format 5 and local closed-backend recovery |
| App removal and future sound | Missing beat stays absent until Undo restores it | Tested numerically | Both channels match independent sample sums. Owner listening remains open. |

The listening file uses generated low kicks and high tones. The erased kick falls on the half-second beat of a one-bar loop.
Undo returns that kick for later loops without changing the high-tone rhythm. Test level, timing, clicks, silence and both channels against a reference.
Owner listening, physical-controller acceptance and active-erasure callback timing remain separate.

The 12 App erasure tests pass. The required check passes 914 App tests and 24 notice tests, then fails on 50 uncovered scenarios.
The engine, TUI and documentation suites pass separately. Main-folder release build and all 18 OpenSpec items pass.
The latest benchmark stays below 50 percent, but it does not explain earlier overruns or measure active erasure.
All implementation boxes remain open. Recording bindings, the panel, active Undo and new indications remain disabled or absent.
Detailed evidence stays in `target/97-app-erasure/results.log`.

### Thirteenth implementation boundary: erasure-reader completion

The owner approved all remaining change-97 plans and requested a complete usable recording feature.
Continue without another approval request for each existing boundary. Keep publication, live devices and owner projects outside this permission.
First test App collection while the engine closes erasure intervals and removes rows concurrently.
A failed bounded feedback read must defer interval consumption, row confirmation and pass acknowledgement until the complete accepted prefix is available.
Use the same collector for production and tests. A private feedback-reader boundary permits deterministic unavailable-read tests without changing engine atomics.
Also test real concurrent engine writes and stable retry after the writer stops. Keep unrelated rows, raw order and one-pass Undo intact.
This boundary changes App `pattern_record.rs`, this design and the task record. It adds no sound behavior, binding or saved field.

### Fourteenth implementation boundary: global and active-pass Undo

Add `pattern.undo` to GLOBAL actions and bind Alt+Z in the shipped map.
Explicit owner entries retain priority. Command releases and repeats do not request another Undo.
Connect the action through production keyboard, raw note and learned CC dispatch on every screen.
Pads Ctrl+Z first finishes and cancels an active pass. Idle common history keeps its existing order.
Local Chop Undo remains separate. Pattern Undo never changes local cuts or the current screen.

An active Undo request retains the matching pre-pass snapshot until collection reaches the complete accepted boundary.
Finish the pass with its existing identity. Close pending Gates and collect erasure before restoration.
Queue final publication and acknowledgement before the restored pattern. Preserve that order when the command queue fills.
Keep autosave blocked until the next collection turn restores the snapshot.
Restore only pattern rows and settings. Do not add a completed-pass history step for this cancellation.
An empty pass or cancelled count-in must not undo an earlier edit.
Audio loss completes the same cancellation locally, without another callback or audio command.
Idle pattern Undo requires a pattern pass at the top of common history.
A newer pad or effect edit refuses pattern Undo with instructions to undo that edit normally.
No redo, new saved field, recording-start binding or panel belongs to this boundary.

Files: App `pattern_record.rs`, `dispatch.rs`, `main.rs`, pattern input tests, input `lib.rs`, `config/keymap.toml`, design and tasks.
The exhaustive help lists in `main.rs` must describe the new action and the absence of redo.
The modifier line uses the short label `pattern undo`. Keep the exact help-list test and add the new command to its expectation.
Reject a separate pattern history because restoration could cross a newer pad swap.
Reject immediate restoration before collection because late feedback could change the restored take.

| Crossing | Required result | Status before implementation | Check |
| --- | --- | --- | --- |
| Active Undo and delayed Gate completion | Finish once, restore the pre-pass take | Open | Production input and accepted terminal state |
| Empty pass and older history | Preserve the older edit | Open | Count-in, empty recording and repeated requests |
| Undo and full command queue | Publication, acknowledgement, restoration keep their order | Open | Existing bounded production queue |
| Undo and audio loss | Restore locally without another callback | Open | Stopped isolated backend |
| Global Undo and newer swap | Refuse without changing row addresses | Open | Production swap and ordinary Undo |
| Global Undo and local Chop edits | Preserve local cuts and screen | Open | All screens and three input sources |
| Text, questions and learning | Preserve existing input ownership | Open | Production dispatch guards |

### Fifteenth implementation boundary: recording commands and real Chop pads

Connect GLOBAL `pattern.record` and `pattern.erase`, with Ctrl+R and Alt+E defaults.
Keep explicit owner entries, note and CC learning, release ownership and source-specific input priority.
Remove only the obsolete Ctrl+R microphone notice. Add hidden-default warnings for the new keys.
Use the existing capture collector and audio commands, not a second recording state machine.
A pending start already owns its pass. Another press ends that same pass before audio feedback arrives.
A pending end cannot become a new start. A new pass needs an accepted Idle boundary and a fresh identity.
Retain erasure command intent per pass so two presses before a callback still mean On then Off.
Accepted audio feedback remains authoritative for phase and visible recording state.

Refuse pattern start before A1 creation while microphone or output capture is active.
CountIn, Recording, Finishing and pending completion refuse new microphone or output capture.
Name the current target and how to finish it. A rejected microphone release cannot stop another target.
A conflicting recording request temporarily exposes its Warn notice ahead of the audio-capture duration line.
Use a typed refusal marker, not a text match. Keep earlier Error priority and normal acknowledgement.
Other notices keep their existing recording precedence. Preserve hidden Info time when the refusal expires.
Test the visible refusal, not only its history entry. Keep the persistent recording status unchanged.
Refuse start with placement, marking or Chop assignment active. While capture owns pads, refuse new placement and marking.
Unclaimed Esc finishes capture only after existing higher-priority overlays close.

During capture, Chop routes new presses to real pads, including empty pads, in the selected absolute bank.
Publish that routing choice in the existing packed MIDI focus snapshot. This is not a saved format change.
Draw the real grid, preserve preview controls and do not hush pads when entering Chop during capture.
Keep the original release address when recording ends or the bank or screen changes.
The compact controls panel, parameter knobs, full phase feedback, final WAV and performance measurement remain later work.
This boundary adds no dependency, factory controller assignment or redo.
It does not access owner projects or publish changes.

Files: App `pattern_record.rs`, `dispatch.rs`, `main.rs`, `view.rs`, `notices.rs`, pattern input tests, input `lib.rs`, keymap, design and tasks.
The packed focus bit also requires mechanical updates to explicit snapshots in the App MIDI and shared-input tests.
A full audio-command queue already accepts a pending start through ordered UI commands.
Preserve that behavior and its autosave test. Queue pressure alone is not a refused start.
After accepted Idle feedback, do not retain the old Finishing pad route until another collection turn.
Reject deriving every toggle from the last redraw because repeated commands can precede a callback.
Reject routing all Chop hits through slice preview because those hits cannot address real recording pads.

| Crossing | Required result | Status before implementation | Check |
| --- | --- | --- | --- |
| Start, second press and delayed callback | End the same pending pass, never start a second pass | Open | Production input before processing audio |
| Erasure On, Off and delayed callback | Preserve both ordered commands | Open | Raw MIDI and keyboard with no intervening callback |
| Audio capture and pattern start | Refuse without creating A1 or changing the current target | Open | Both audio sources, press and release |
| Capture and placement or marking | Refuse the incompatible request before consuming a pad | Open | Both request orders and production dispatch |
| Chop entry, bank change and held release | Keep sound and the original release address | Open | Keyboard, raw note and CC pad input |
| Text, questions, learn and restoration | Keep input ownership and reject underlying commands | Open | Production guards and earlier releases |
| Full command queue and start or end | Retain accepted pending commands and block autosave until completion | Open | Bounded production queues |
| Old project and new pass | Reject old commands and retain original permitted releases | Existing protocol, route checks open | Tagged production dispatch |

At review, targeted tests cover pending starts, repeated ends, rapid erasure toggles and both audio-target conflicts.
They also cover placement requests, questions, text, learning, restoration, real Chop banks, Gate releases and separate previews.
The question test keeps confirmation ahead of the new placement guard.
The visible-refusal test checks six-second warning time, paused Info time and an earlier Error before the refusal.
One pad input reads its recording route once, so a concurrent phase change cannot split that decision between preview and real pads.
These are command-boundary checks, not complete feature acceptance.
Compact controls, the combined raw erasure burst, stale input across passes and dedicated Esc priority checks remain open.

### Sixteenth implementation boundary: absolute pattern parameters

First connect the three named MIDI knobs. Reuse their setting path for the later compact panel.
This boundary does not open the panel or add its button navigation.
The existing parameter lists and GLOBAL knob requirement define the behavior.
No new format, factory assignment, dependency, screen or Undo category is necessary.

Files:

- Input `midi.rs` and `lib.rs`.
- App `pattern_record.rs`, `dispatch.rs`, `main.rs`, `values.rs` and pattern input tests.
- Design and tasks.
Before-files, hashes and preflight results stay in `target/97-pattern-parameters/`.

Use one setting descriptor for Bars, Quantize and Strength. Keep their ordered values in the existing specification order.
Map a finite absolute position to its nearest permitted index, with ties later. Clamp finite positions to the permitted range.
Keep jump, pickup and scale in the existing per-device and per-control takeover model.
Do not create A1 while pickup waits or input ownership blocks the command.
An accepted edit without A1 creates the existing empty two-bar, 1/16, strength-100 default before changing that setting.
Keep raw rows, their order and runtime identities. Do not add Idle setting edits to recording-pass history.
A pass still restores its complete pre-pass settings through the existing Undo contract.

Flush accepted capture before an edit. Use the collector's coherent current-project boundary, not a newer uncollected sequence.
Refuse pending starts, CountIn, Finishing, pending end/Stop and incomplete erasure collection.
During Recording, refuse Bars and allow grid or strength without restarting transport.
A queued Stop retains end intent before callback feedback, without claiming an accepted phase early.
After audio loss, allow Idle settings only after retained capture and active Undo complete locally.

The backend rejects an unstamped recorded-pattern publication after erasure starts.
`Capture::accepts_publication` checks pass and sequence. Existing grid-publication and retained-window tests pass before this boundary.
Publish Recording edits with that accepted pass and sequence. Keep existing retry collection if newer erasure makes a stamp stale.
An Idle edit uses ordinary recorded-pattern publication. Do not change the callback or release rules.
A bounded full command queue retains publication order. Offline edits do not queue commands for a stopped callback.

Observe the applied saved value through the existing value popup.
Pickup waiting and phase refusal must not refresh a popup with an unapplied position.
External setting changes, project replacement and Undo restoration reset the relevant takeover targets.
Use distinct fixed targets for the three settings, separate from tempo, master and pad targets.
Reject a new takeover dispatcher: the existing one already supports the required three responses.

| Crossing | Expected result | Status before implementation | Check |
| --- | --- | --- | --- |
| 0, 64, 127 and every discrete boundary | Exact lists, ties later, no raw rewrite | Closed | Raw MIDI on all screens and all 128 positions |
| Two devices, keyboard-equivalent external edit and Undo | Independent response state catches the current value | Open for keyboard routing | Pickup, scale, reset helper and both Undo paths pass. Panel tests remain |
| Pending start/end/Stop and full queue | No edit enters a closing pass | Closed | Phase refusals before callback. Full-queue publications and end retain order |
| Recording and held erase target | Apply stamped grid change without reviving erased rows | Closed | Raw knob audio checks and stale publication retry after erasure advances |
| Missing A1, same value and pickup waiting | Create only on accepted input. No false popup or dirty step | Closed | State, command, dirty-time and popup assertions |
| Text, picker, learning, question and restoration | Input ownership wins without a pattern side effect | Closed for direct knobs | Eight blocked states retain input ownership. Panel routing remains later |
| Audio loss or project replacement | Retain the take. No stale takeover ownership | Closed | Project reset and interrupted Gate recovery with subsequent offline editing |
| Nonfinite position or compilation refusal | Keep the original pattern and settings | Closed | Nonfinite rejection, clamped finite values and invalid raw-state checks |

This boundary can cover direct knob ranges. It cannot claim complete keyboard-panel or learned-button scenario coverage.
Keep the final recording WAV, full catalogue and active recording performance tasks open.

### Seventeenth implementation boundary: compact pattern controls

Connect Alt+N and the five GLOBAL control buttons through the existing input routes.
Keep the three setting setters from the previous boundary. Do not add another parameter model.
The panel uses the existing edit-key layer for keyboard directions and Enter.
Translate its parameter and value actions only after text, picker, question and learning ownership checks.
Explicit edit-key overrides and `none` remain effective. MIDI uses the named pattern buttons.
Parameter and value buttons require the panel. Direct parameter knobs remain independent of it.
Opening without A1 creates the existing default only after input ownership permits the request.
Opening an existing A1 during CountIn or Finishing permits inspection, not setting edits.
Enter, Esc and the effective controls toggle close the panel before an unclaimed recording end.
Closing restores the underlying edit state, selection, bank and screen without a second action.
A screen switch closes the panel through the existing screen transition.
On Chop, the open panel shows and plays real assigned pads, as the existing Chop delta requires.
Keep this route separate from active-pass ownership. Opening controls cannot finish a pass or block its start.
Closing controls restores the slice keyboard outside capture. Earlier releases retain their original real-pad address.
External button edits reset takeover for the changed parameter. Refusals do not change pickup ownership.

Draw seven compact rows in the existing right column on screens with pads.
Do not cover or move the pad grid, title, status or safety notices.
FX and Settings use the corresponding right-side content area.
Show three values, the selected row, phase restrictions and effective navigation and close keys.
Hide this drawing behind questions, text, pickers, learning and full help.
Keep it in the graphics blocker list so a wave image cannot cover its text.
The offline catalogue uses the same panel snapshot and drawing functions as the live frame.
Full accepted phase, count digits and erasure markers remain the next boundary.

Files: keymap, input `lib.rs`, App `main.rs`, `dispatch.rs`, `pattern_record.rs`, `values.rs`, `view.rs`, `screens.rs`, pattern input tests, TUI `lib.rs`, design and tasks.
Before-files and hashes stay in `target/97-pattern-panel/`.
No callback, dependency, saved format, factory assignment, owner project or publication changes belong here.
Reject a new modal keymap format: the existing edit layer already supplies the required keys and owner overrides.
Reject a centered dialog because it can cover playable pads at 80 by 26.

| Crossing | Required result | Status after software checks | Check |
| --- | --- | --- | --- |
| Closed controls and value button | Refuse without creating A1 or editing the screen | Closed for tested input | Raw note and CC buttons |
| Panel, Enter or Esc and active recording | Close only the panel. A later unclaimed Esc ends the pass | Closed for tested input | Production keyboard and MIDI |
| Panel and text, question, picker or learning | Higher-priority input retains ownership and earlier releases survive | Closed for tested input | Production input sequences |
| Keyboard edit and two pickup devices | Both devices catch the new saved value independently | Closed for tested input | Actual panel keys between raw knob movements |
| Length limits and hidden rows | Clamp without rewriting raw rows or runtime identities | Closed for tested input | Both limits and recording refusal |
| Supported sizes, hidden pads and safety warnings | Readable controls without a shifted or covered pad grid | Closed for offline drawing | Shared renderer and catalogue buffers. No live terminal check |
| Repeated toggle or command release | No second toggle or sounding pad | Closed for tested input | Layouts, protocols, raw note and CC releases |
| Unassigned Chop, controls and a later release | Play real pads only during pattern work. Keep the original release | Closed for tested input | All input sources, banks and capture phases |

### Eighteenth implementation boundary: accepted recording feedback

Show accepted count-in, recording, erasure and finishing on every current screen.
Read one coherent audio snapshot for each frame. Use it for the title, pad targets and phase help.
Do not infer acceptance from a queued command. Recheck project readiness around the snapshot read.
Use existing badges and help rows. Do not add a screen, a step grid or a persistent field.
Show COUNT with the remaining beat, REC A1 with bar, beat and length, and a distinct ERASE state.
Keep grid and strength in the normal recording label and in F1 when title space requires a compact label.
The erase label names the first actual target and the number of additional targets.
F1 lists every target with bank and pad identity, including targets outside the visible bank.
This keeps identities accessible with a hidden grid without covering questions or safety notices.
Give each visible erase target an E corner. Suppress its ordinary hit flash, not another pad's sound indicator.
Add the Erase variant to the existing public PadCorner drawing model. Keep its other variants unchanged.
The live frame and catalogue pass the same snapshot through the existing formatter and drawing paths.
Catalogue recording fixtures use the offline audio engine to accept commands, not fabricated UI phases.
Test pending commands, all four count beats, tick-zero recording, erase entry and exit, cross-bank targets and a replaced project.
Retain existing audio-recording and safety indicators. Pattern recording does not claim microphone or output capture.
F1 explains manual-save completion and the separate no-release microphone and erase fallbacks.
Physical input and listening remain unverified. Active recording performance remains a later boundary.

Files: App pattern_record.rs, view.rs, main.rs, screens.rs, input_tests/patterns.rs, TUI lib.rs, design and tasks.
Before-files and hashes stay in target/97-recording-feedback/.
Reject a new UI phase timer because it could disagree with audio-time count-in and erasure expiry.
Reject a new centered overlay because it would cover playable pads and compete with questions.

| Crossing | Expected result | Current status | Check |
| --- | --- | --- | --- |
| Pending start and delayed callback | No premature COUNT or REC label | Closed in software tests | Keyboard, raw notes and learned CC commands before and after audio acceptance |
| Count beat and tick zero without UI input | Accepted digits and position advance | Closed in software tests | Four audio-timed counts, tick zero and the next beat at both sizes |
| Erasure, banks and hidden pads | Actual targets remain identifiable without a sounding flash. | Closed in software tests. | Silent targets, releases, E corners, legacy expiry and all 128 F1 target identities pass. |
| Project readiness and stale feedback | No previous-project recording state | Closed for replacement | Session replacement rejects the earlier snapshot before the callback accepts the new epoch |
| Narrow title, long names and safety | Keep recording identity and fixed safety rows | Open for long names | Both sizes and offline safety combinations pass. Long-name recording checks remain |
| Panel, learning and questions | Keep higher-priority ownership and effective controls | Partly closed | Phase and overlay catalogue checks pass. Existing learning guards remain |
| Owner edit-layer bindings | Compact help names only effective commands | Open | Global overrides pass. Edit-layer overrides need a separate check |
| F1 length and width | Every required instruction remains reachable. | Closed in software tests. | All wrapped rows are visible through page navigation on every current screen at both sizes. |

### Approved F1 readability boundary

The owner approved this boundary. Amend the keyboard and screen-help deltas, this design and tasks before code.
Wrap long key-table rows without losing text. Add Alt+PageUp/PageDown for previous and next pages inside F1.
Add GLOBAL `help.page_prev` and `help.page_next` commands so the owner can learn controller buttons.
Keep ordinary bank keys, pad releases, owner overrides and the existing Messages controls unchanged.
Keep accepted recording feedback visible while help is open. Clamp the page after resize or changing content.
Outside F1, the new default page commands do not change the current screen.
Extend the input action registry and TUI help page model. Keep saved project formats unchanged.
No factory controller assignment, dependency, audio processing or input-gain change belongs to this boundary.
Reject reordering alone because new rows would displace other commands without making the full list reachable.
Long help requires more than one page. Physical Alt+Page reporting remains unverified.

Keep independent offsets for the key table and Messages. Reopening F1 starts at the first key-table row.
Use the same wrapped lines for page bounds and drawing. A page command moves by the visible text height.
Clamp to the last full viewport, not to an empty page. Show the effective previous and next keys with the current page position.
Keep the accepted recording caption above either help tab. Full erase identities remain reachable in the key table.
The caption and key rows use the same accepted recording snapshot as the rest of the frame.
Retain page requests that arrive before the first frame, rather than applying unknown bounds.
Apply them in order after wrapping. A later tab change does not cancel an earlier key-table request.
Closing help discards queued requests. The UI owns this short-lived queue. The audio callback never reads it.

| Crossing | Expected result | Status after focused checks |
| --- | --- | --- |
| Long descriptions, wide characters and narrow frames | No lost key or description text | Closed. Unicode column and all-row render checks pass |
| Page repeats, releases and owner edit bindings | Repeat navigation only. Releases retain their sound ownership | Closed. Both protocols, layouts and active edit overrides pass |
| Help opening and page commands before the first frame | Retain ordered page requests until wrapped bounds exist. Closing discards them | Closed. Keyboard, raw notes and learned CC batches pass |
| Messages, resize and changing phase | Independent offsets, clamped pages and accepted recording caption | Closed. Both tabs, sizes and actual erase-list removal pass |
| Questions, text and learning | No underlying page command or recording change | Closed. Blocked page commands and audible Gate releases pass |
| Ordinary modifier help | No unavailable F1-only page commands | Closed. The original Alt-row comparison passes without assertion changes |
| No factory assignment or device access | Learned software controls only | Closed by scope. Physical controls remain unverified |

Approved file set:

- `config/keymap.toml`, `crates/input/src/lib.rs` and `crates/input/src/midi.rs`.
- `crates/app/src/help.rs`, `crates/app/src/dispatch.rs`, `crates/app/src/midi.rs`, `crates/app/src/main.rs` and `crates/app/src/view.rs`.
- `crates/app/src/screens.rs`, `crates/app/src/input_tests/patterns.rs` and `crates/tui/src/lib.rs`.
- `openspec/changes/97-pattern-record/specs/keyboard/spec.md` and `openspec/changes/97-pattern-record/specs/screen-help/spec.md`.
- `openspec/changes/97-pattern-record/design.md` and `openspec/changes/97-pattern-record/tasks.md`.

### Complete raw MIDI erasure workflow checks

Tasks 4.3 and 5.1 need a complete production-input check, not only isolated engine intervals.
Use raw notes and learned CC buttons on every current screen. Start erasure before its following target press in one input batch.
Advance accepted audio for 6000 frames at 120 BPM, then release at tick 240.
Leave the UI unserviced for three loops after that release. The erasure-off command can remain queued during this delay.
Do not replace the direct MIDI release with a later UI command.

Use raw target rows at ticks 0, 239 and 240 with quantize Off.
The half-open interval removes the first two rows, not the release-boundary row.
Retain a different pad and a hidden row. Compare both sound channels against an independent manual reference.
No collected notification may produce a new recorded onset or revive the removed rows. One pass Undo must restore the original raw order.
This check adds no behavior or factory assignment. Keep the required scenario audit unchanged.

| Crossing | Expected result | Status after diagnosis |
| --- | --- | --- |
| Erasure command and target in one raw batch | Accept erasure first. No manual target sound or capture | Start and the first silent interval pass |
| Release after a deferred press | End the target at the accepted release, not the redraw | Reproduced failure for notes and CC |
| Tick 239, release tick 240, another pad and hidden rows | Remove only traversed target rows | The release-boundary row is also removed. Complete row checks remain blocked |
| Delayed collection and another loop | No revived notes or additional captured rows | Not reached after the release failure |
| End and pass Undo | One history step restores raw rows in their original order | Not reached after the release failure |

Files: `crates/app/src/input_tests/patterns.rs`, this design and `tasks.md`.
Preserve before-files and literal results in `target/97-erase-workflows/`.
Reject a larger feedback queue or test-only routing because neither proves the production input order.
If this check exposes a missing implementation, diagnose it before choosing a fix or expanding the file set.

The initial complete workflow failed at the release assertion. A smaller check reproduced the defect without sounds, saved files or an erasure-off command.
A preceding unconsumed command makes MIDI retain the press as a UI hold in `crates/app/src/midi.rs`.
Its release sends only `UiPadRelease`. App sends the audio release later in `crates/app/src/dispatch.rs`.
The MIDI acknowledgement currently publishes focus and input sequence, not the accepted hold's release route.
If App consumes the erasure command before the press, release instead uses the audio queue and clears the target.
Both note-release encodings and learned CC show the same contrast.
With two rows at ticks 0 and 240, the deferred case removes both. The control removes only the first row.
The first 128 audio frames after release expose the extra removal. The UI remains unserviced throughout that interval.
Two permanent tests retain these checks. They now pass through the complete stereo reference, three-loop delay and pass Undo.
The initial failures remain in `target/97-erase-workflows/`. Fix results remain in `target/97-erase-release/`.

#### Approved release fix

The owner approved this ten-file extension and requested completion of recording and the separate callback investigation.
Approval covers implementation, not commits, publication, archive or live devices.
Use this file set for the erasure-release fix:

- `crates/app/src/midi.rs`, `crates/app/src/dispatch.rs` and `crates/app/src/main.rs`.
- `crates/app/src/input_tests/patterns.rs`.
- `crates/engine/src/lib.rs`, `crates/engine/src/rt.rs` and `crates/engine/src/capture.rs`, if the release needs a bounded runtime identity.
- `crates/engine/tests/realtime_no_alloc.rs` for any callback change.
- `openspec/changes/97-pattern-record/design.md` and `openspec/changes/97-pattern-record/tasks.md`.

Use the actual accepted pad and original project identity, not the later focus or captured bank guess.
Release must not wait for another UI frame. Preserve command order, repeated presses, multiple ports and channels, disconnect and full-queue retries.
Check press and release before one callback as well as release after accepted audio advances.
Before a runtime-only engine command extension, specify its fields and ordering here.
Do not change existing commands, saved formats, factory assignments, ordinary manual release semantics or audio-input gain.
Keep callback work bounded with no allocation, deallocation, locks or waits.
Reject a blind release for the captured pad because earlier deferred bank commands can change the accepted pad.
Reject increasing the feedback queue because it does not change release ownership.
The former stop pending scope approval no longer applies.

##### Accepted release protocol

MIDI retains one atomic release signal per original UI hold, with the existing port, channel, key and project identity.
A release or disconnect sets that signal before sending its UI notification. The signal never waits for a command queue.
The UI receives the signal in the original input envelope. It resolves the absolute pad after preceding deferred commands.
Only pattern work creates a separate accepted-hold handle. This handle shares the original signal and retains its own erasure disposition.
Ordinary input outside pattern work keeps the existing commands and release route.

Add two runtime-only commands:

- `PatternPadPress { pass, pad, velocity, hold }` carries the intended pass, resolved absolute pad, velocity and shared accepted-hold handle.
- `PatternPadRelease { pad, hold }` carries that same accepted handle and pad, not the current bank.

Both commands retain the existing project epoch, readiness guards and UI queue order.
A press for a different accepted pass cannot enter that pass.
The engine decides whether the press is a silent target or ordinary sound at the command boundary.
An erasure press marks its handle before any later release command can run.
A pending release must not consume the UI sound-release obligation of a later press.
Only an accepted ordinary handle consumes that obligation. The ordered engine command still resolves a pending release after its press.
A release command for that marked handle cannot cut a later manual note. An ordinary handle retains the existing per-pad release behavior.
A repeated accepted press keeps its own disposition. Rejected presses cannot replace an accepted release obligation.

The engine retains at most 4096 active erasure handles in storage allocated during setup.
Before commands and before audio rendering, it checks the retained signals with bounded atomic reads.
A released handle closes only its original accepted pass and absolute target at the current audio boundary.
A release already present at press acceptance closes a zero-length interval before sound rendering.
Multiple holds on one pad keep the existing per-pad release rule: the first permitted release closes that pad's target.
Each target retains its original generation through grid publication. A handle cannot address a later target on the same pad.
Erasure-off, end, Stop and project replacement invalidate the original target.
Storage exhaustion refuses a new silent target and increments the existing erasure-refusal warning count.

Return handles through the existing retirement queue. Reserve space before discarding an incoming handle or starting ordinary sound.
A full retirement queue may retain a closed handle, but cannot extend its erase interval.
No callback allocates, frees the last reference, waits, locks or enlarges a queue.
Reject a direct release based on the captured bank. Reject a general redesign of ordinary manual holds.

| Crossing | Expected result | Status after focused software tests |
| --- | --- | --- |
| Deferred command and release after audio acceptance | Close at the release boundary without UI work | Closed. Both original tests pass, including stereo reference, delayed loops and Undo. |
| Release before UI acceptance or before the first callback | Retain the signal and close a zero-length target | Closed. Notes and CC cover both orders, both note encodings and disconnect. |
| Earlier bank command and later bank or screen changes | Release only the resolved absolute pad | Closed. The accepted pads are 16 and 17, not the originally captured bank. |
| Two ports or channels and a repeated physical control | Keep accepted holds separate without changing per-pad release rules | Closed. Wrong-source releases retain the independent target. |
| Erasure-off, fresh Gate and late old UI notification | The old erasure handle cannot cut the fresh sound | Closed before and after acceptance. A separate pending-release regression retains the fresh Gate's own release. |
| Full command or retirement queues | Retry ordered work without extending a released target or freeing in audio | Closed. Early and late signals work with both queues full. |
| Grid publication, next pass and project replacement | Keep the original hold or invalidate it, never address a new pass | Closed. Notes and CC retain current targets through old signals, envelopes and release notifications. |
| Callback memory operations | No allocation or deallocation for accept, close, discard or reset | Closed. All 28 allocator tests pass, including 4096 handles and full retirement. |

Preserve before-files, hashes, the full bounded diff and literal results in `target/97-erase-release/`.
The separate callback investigation must retain the original workload and all historical maximum overruns.

Additional active workloads use 20000 blocks of 128 frames at 48000 Hz and 172 BPM.
They retain 32 long manual voices, pitch, envelopes, filter, drive, both sends, reverb and delay.
The original playback, dense-pattern and project-switch workloads remain unchanged.

- Active recording accepts a manual onset every 64 blocks. A silent Gate on pad 127 starts every 128 blocks and releases one block later. Shared accepted-hold commands exercise command acceptance and retirement. Drain feedback and garbage outside the measured callback. Check accepted onset counts, completed gates, phase and all loss counters.
The constructor supplies a present zero-length sample to unused pads. That sample can briefly consume a voice before it ends.
Explicitly set the silent Gate pad's sound to None. Do not weaken the 32-voice check or change capture limits.
The full recording run must accept 470 onsets and complete 157 Gates with all 32 sounding pads present.

- Active erasure starts with 4096 compiled rows and 128 held targets. The original 32 manual voices keep playing. Retain the raw snapshot to represent delayed UI removal. Check all targets, suppression, erased rows and loss counters. Measure release and the accepted Finishing boundary separately, without hiding them in steady-state percentiles. UI reconstruction and acknowledgement are not part of this callback workload.
- Setup, allocations, queue preparation, feedback reads, garbage destruction, report formatting and CSV writes stay outside measured sound processing.
- Use the existing paired wall/thread CPU clocks and trace identities for every active phase. Report mean, p99, maximum and each count above 50 percent. The new active workloads also fail if any maximum exceeds 50 percent, or if a workload invariant fails.
- Preserve the historical 61.07 percent wall maximum as unresolved unless matching measurements can identify its cause. A new observed scheduling gap explains only that new observation, not the historical block.
Measure active recording and erasure before claiming their callback budget. Do not replace maximum results with p99.

### Proposed completion file boundary

The owner approved this nine-file extension with an explicit go.
The ten-file release fix does not complete recording or the callback investigation.
At approval, the complete audit listed 19 uncovered scenarios and the final demonstration WAV was absent.
The completion tests now cover every active scenario and generate the final WAV. The callback maximum limit remains open.
Approval covers this implementation boundary, not commits, publication, archiving or live devices.
Keep the current ten-file set. Add these files only for existing change-97 requirements:

- `crates/app/src/view.rs` and `crates/app/src/screens.rs` for effective help, long names and visible safety checks.
- `crates/engine/benches/callback.rs`, `crates/engine/benches/support/mod.rs` and new `crates/engine/benches/support/recording.rs` for separate recording and erasure workloads.
- `crates/engine/tests/benchmark_report.rs` for workload and report checks.
- `README.md` and `docs/LIVE-CHECKS.md` for exact keyboard, learned-controller and listening procedures.
- Generated `renders/97-pattern-record.wav` from the production-path App test.

First complete substantive scenario checks in the existing App test file. Do not add name-only associations.
Then check visible state and produce the WAV against independent timing and stereo references.
Keep the original callback workloads, clocks and exit checks unchanged. Add separate active recording and erasure measurements.
Retain every measured maximum, including the historical 61.07 percent overrun. A passing p99 cannot resolve a maximum failure.
Finish with required checks and a normal main-folder release build. Keep hardware and listening acceptance separate.

Reject a larger feedback queue, a test-only input route or weaker assertions as substitutes for missing behavior.
Add no dependency, saved format, factory assignment, input gain, live-device action or owner-project edit.
If a measured fault needs a file outside this set or a new behavior decision, stop and propose the exact extension.
Preserve the verified release fix. Save before-files, hashes and literal completion results in `target/97-completion/`.

#### Effective short help and safety crossings

Short pattern help and the compact controls use the same effective-binding lookup as F1.
That lookup includes the active edit layer and explicit `none` entries. It converts symbolic key labels to config key names before lookup.
When no effective key remains, show `none` with the action instead of hiding its recovery role.
Do not show a shadowed default as a working key. Do not change event routing or bindings.

Render real phase refusals, newer-Undo refusals, row-limit warnings and feedback overflow with persistent save and audio status.
Use long project names, the real accepted recording state, every current screen and both catalogue sizes.
Keep row positions and safety status visible with controls open or closed. Compare cells outside the overlay.
The catalogue must use the App project name when one exists. Retain its ordinary fallback name otherwise.

#### Completion acceptance sequences

The final App workflow uses the default 120 BPM and two-bar A1 at 48000 Hz.
It starts without A1 through real keyboard, note and learned CC commands. Count-in completes before the demonstration starts.
Generated 55 Hz and 110 Hz sounds last 2400 frames. Each right channel uses 1.5 times its left frequency.
A squared sine envelope starts and ends at zero. The peak scale is 0.05.

The 28-second demonstration uses these boundaries, relative to accepted A1 tick zero:

- First manual hits: frames 5040 and 36000. The first quantized repeats start at 198000 and 228000.
- Overdub starts at 384000. Its new high hit starts manually at 408000 and repeats at 600000.
- Erasure starts at 768000. It suppresses the low hit at 774000 and ends before a new manual low hit at 800000.
- That pass also changes the grid to 1/8 and strength to 50. Its new low hit repeats at 994000.
- One Undo at 1008000 restores the pre-pass rows and settings. The original low hit returns at 1158000.
- The file ends at 1344000. An additional empty pass creates no history step.

An independent manual engine renders the specified accepted onsets without a pattern or capture model.
Compare every sample in both channels within 0.000001. Check finite samples, clipping, silence, starts within one frame and maximum adjacent-sample change.
Compare RMS in each onset window against that independent reference within 0.01 dB.
Write stereo float32 WAV only after those checks pass. Keep listening and physical controller acceptance unverified.

| Crossing | Required result | Completion test result |
| --- | --- | --- |
| First recording and stopped transport | Default A1, one count-in and no changed pad sounds | Closed by the three-source final workflow |
| Overdub during playback | Preserve clock, old rows and separate pass history | Closed by the three-source final workflow |
| Erasure, grid changes and new hits in one pass | One Undo restores all pre-pass pattern data | Closed by the stereo reference and history checks |
| Empty pass after Undo | No extra history step or restored unrelated edit | Closed by the final workflow |
| Owner file loading and repeated startup | Keep exact bytes and effective bindings | Closed by the extended production loader test |
| Text, questions, pickers and learning | Block every recording command, but release earlier sound | Closed by the extended raw-input and Gate test |
| Placement before or during recording | Refuse without replacing or marking a pad | Closed by the extended pad-snapshot test |
| Audio loss before or after accepted finishing | Save complete recoverable gates without fabricated acknowledgement | Closed by keyboard, note and CC save/reload tests |

#### Completion performance result and limit

The ordinary required run passes all software tests and both new active maximum gates.
Separate process-affinity diagnostics retain the exact workloads, clocks and exit rules. They change no global scheduling or frequency setting.
The CPU 12 diagnostic fails the active erasure maximum gate: CPU reaches 62.64 percent, with ten blocks above 50 percent.
CPU 0 also shows original steady-playback overruns, although its unchanged p99 exit gate passes.
A passing ordinary run therefore does not establish the maximum-budget requirement across the measured conditions.

Matching historical traces distinguish elapsed-time gaps from blocks with CPU time itself above target.
The original 61.07 percent block has a large gap and involuntary scheduling in its larger observation envelope.
Other traces have CPU overruns with almost no gap. Later frequency readings remain observations, not callback frequency traces or proven causes.
The pinned comparison does not establish the precise cause of any older block.

The profiler preflight fails because performance monitoring is restricted. No privileges, kernel settings or audio-device settings were changed.
No timing optimization is selected. Keep this failed acceptance visible and obtain permitted profiling before choosing a runtime change.
All original and diagnostic logs and paired traces remain local. Physical and listening checks remain separate.

### Zoom while editing slice bounds

The owner requested zoom without releasing Ctrl or Alt and approved four Chop aliases.
Ctrl/Alt+I zooms in. Ctrl/Alt+K zooms out. Existing J/L bound controls remain.
Keep Ctrl+Up/Down and their Shift variants for tempo.
Use four screen bindings to the existing zoom actions, not a new modifier fallback or input mode.
The existing dispatcher already repeats zoom and preserves source-specific input ownership.
The edit layer has no conflicting modified I/K entries. Explicit owner overrides retain priority.

The installed terminal parser maps a legacy tab byte to Tab, not a distinct Ctrl+I event.
Keep Tab unchanged and document Alt+I as the legacy replacement.
Tests of distinct Ctrl+I events do not establish that a legacy terminal can report that chord.
No audio processing, MIDI assignment, saved format or pad binding changes are necessary.
The bound-edit test uses the production keyboard handler, not action calls alone.
It keeps the modifier held across a bound move, repeated zoom and another bound move.
Check both bounds, layouts and slice-edit states. Keep assignment and text ownership tests.
Effective modifier and F1 help use the existing map. Test owner removals and remaps.
The short zoom row keeps plain keys. Adding all aliases there hides the plain hint under modifier filtering and exceeds the legacy help width.
Both existing help tests pass after this fix. The full run passes all ordinary App tests but still fails the active recording-scenario audit.

Files: keymap, input key tests, App keyboard and help tests in `main.rs`, keyboard, chopping and help deltas, design and tasks.
Before-files, hashes and results stay in `target/97-bound-zoom/`.
Reject changing Ctrl+Up/Down because that would replace the existing tempo controls.
Reject a general Ctrl/Alt fallback because it could execute unrelated screen commands.
The current panel work remains separate and uncommitted.
Its overlay check now compares identical real-pad roles, not the intended selection and dimming change at panel entry.
All outside-overlay cells still require full equality. Both open and closed frames retain their fixed-row checks.

| Crossing | Expected result | Status after focused tests | Check |
| --- | --- | --- | --- |
| Bound hold, zoom repeat and another bound move | Preserve modifier ownership, cursor and other bound | Closed for tested input | Production keyboard sequence in both layouts and edit states |
| Owner removal or remap | No inherited zoom or misleading zoom hint | Closed for tested input | Map overlay and effective help |
| Legacy Tab and Ctrl+Up/Down | Keep previous Tab and tempo actions | Closed for recorded events | Distinct event tests and legacy Alt zoom. Physical terminal remains unverified |
| Assignment, text or learning | No underlying zoom | Closed for tested input | Production guards with new aliases |

## Risks / Trade-offs

| Risk | Mitigation |
| --- | --- |
| A 1024-frame buffer gives audible live input delay | State the one-buffer bound. Keep timestamp work in 101. |
| Quantize updates or a seam repeat a fresh hit | Use first eligible occurrences and runtime identities, not only a capture-lap flag. |
| A fresh collision suppresses the previous rhythm too early | Select eligible rows before choosing the collision winner. Keep the old eligible hit. |
| Manual playing sounds close to an old same-pad hit | Preserve both accepted sounds. Do not silently introduce replace mode. |
| A lost release changes a later note | Retain bounded terminal gate state with pass and press identity. |
| Erasure acts late after a frozen UI | Track audio-time held intervals and suppress matching onsets in the engine. |
| Undo changes sound edits made during a pass | Store only pattern state in the pass step. |
| Manual save interrupts a continuous overdub | Name this behavior in help. Autosave waits until the pass ends. |
| Chop no longer provides a slice keyboard during recording | Show real pads and a recording-role hint. Restore listen routing afterward. |
| SMC-PAD examples imply implemented holds | Use learned buttons now. Keep factory and physical checks explicitly unverified. |
| Saved Pitch Gate On implies a cut during manual play | Show A1-only scope and a text restriction. Do not infer actual cuts from transport or manual hits. |

## Boundary review

Status means design coverage, not a passed implementation test.

| Crossing | Expected outcome | Status | Planned evidence |
| --- | --- | --- | --- |
| Count-in × cancellation × held pad | No captured warm-up hit or orphan release | Closed in contract | Production keyboard/raw MIDI and exact-frame count-in render |
| Fresh hit × forward quantize × seam | Immediate hit once, then first eligible later occurrence | Closed in contract | Independent manual reference, before/after seam and delayed UI |
| Fresh collision × old eligible row | Old rhythm continues until the fresh row is eligible. Existing winner order then applies. | Closed in contract, tests open | Stronger fresh collision, both raw rows retained, grid separates them later |
| Manual hit × nearby old same-pad hit | Both accepted sounds remain. The fresh row adds no same-lap trigger. | Closed in contract, tests open | Manual hit 20 ms before an old pattern hit, independent two-channel reference |
| Late onset × rounding across zero | Preserve the original capture lap. First repeat does not skip an extra loop. | Closed in contract, tests open | Accepted ticks before, on and after the seam, with delayed publication |
| Gate × release loss × record end | Complete bounded duration, no link to the next pass | Closed in contract | Full feedback queue, retrigger and end-boundary tests |
| Erasure × UI freeze × bank change | Only original absolute pads and acknowledged intervals change | Closed in contract | Raw MIDI held targets across multiple loops |
| Record end × save refusal × retry | Complete in-memory take and Undo remain, no false save success | Closed in contract | Save/reload using production serialization |
| Pattern Undo × newer swap | Refuse a jump over the swap, preserve note addresses | Closed in contract | Production swap/Undo queues and raw rows |
| Chop entry × active recording | Rhythm continues, real pads play, preview remains separate | Closed in contract | Keyboard/raw note/raw CC routing and sound comparison |
| Audio loss × pending finish | Recover known take and permit saving without another callback | Closed in contract | Isolated backend-loss state test, no live device fault |
| Pitch Gate On × manual or mixed playback | Keep A1-only scope visible. Manual sound stays unchanged. | Closed in contract, tests open | Production keyboard and raw MIDI with panel, popup and help assertions |
| Pitch Gate On × source or pitch change | Refresh the text restriction without changing the saved switch. | Closed in contract, tests open | Empty, whole-file, detached and linked sources, then non-positive and positive pitch |
| Pitch Gate explanation × narrow layout × safety warning | Preserve existing rows and readable text without another top badge. | Closed in contract, tests open | Both catalogue sizes, hidden pads and edit open or closed |
| Controller holds × factory profile | No factory tap/hold claim or reset of learned controls | Outside scope | Hardware measurement remains in 91 |
| Chop solo preview outside recording | Keep the existing 96 question visible | Requires separate decision | Do not archive 96 on this design's authority |

Axes cover phase, input source, event order, loop seam, queue pressure, save failure and project/audio lifetime.
Physical comfort remains unknown until the owner uses the device.

### Capture protocol review

This table covers only the first implementation boundary, not the complete recording feature.

| Boundary | Status | Evidence or remaining work |
| --- | --- | --- |
| Old epoch or unready command | Closed by tests | Tagged production queues reject stale starts, ends, acknowledgements and onsets. |
| Repeated or mismatched pass command | Closed by tests | Only a fresh nonzero identity starts. Completion checks identity and final sequence. |
| Full feedback queue and end | Closed by tests | The terminal snapshot survives. Another start cannot replace it before acknowledgement. |
| Feedback loss and later draining | Closed by tests | Separate counters retain losses. Draining never sends omitted events later. |
| Recording pressure and sound | Closed by tests | Both output channels match the control for every sample, with existing scheduled notes. |
| Stop and clock reset | Closed by tests | The end snapshot retains the accepted tick before reset. Manual tails match the control. |
| Empty blocks or concurrent reads | Closed by tests | Empty blocks preserve time. Concurrent readers accept only complete snapshots. |
| Allocation or deallocation | Closed by tests | Allocator counters cover normal capture, full feedback, Stop, acknowledgement and project reset. |
| Lost Gate release and recovery | Closed by tests | Retained terminal slots close accepted gates despite a full queue. Omitted onsets stay omitted. |
| Keyboard, MIDI, App, save and audio loss | Open | Later tasks connect the protocol to owner workflows. |

The new tests are in `crates/engine/tests/pattern_capture.rs` and `crates/engine/tests/realtime_no_alloc.rs`.
The concurrent publication test is in `crates/engine/src/capture.rs`.
These tests do not establish physical-controller or listening acceptance.

### Gate capture review

This review covers the engine boundary of task 1.2, not App publication or erasure.

| Boundary | Status | Evidence or remaining work |
| --- | --- | --- |
| Loop seam and tempo change | Closed by tests | The accepted interval produces 480 ticks. Saved playback matches independent manual commands on both channels. |
| Retrigger and end with a held pad | Closed by tests | Each terminal slot keeps its own duration. Ending capture does not release the manual sound. |
| Zero, midpoint and maximum duration | Closed by tests | Zero intervals become one tick. Midpoints round later. Values above 245760 clamp at that limit. |
| Lost release and refused retrigger | Closed by tests | Queue loss and row-limit refusal still close the preceding accepted Gate. No omitted onset enters recovery. |
| Hidden rows, collisions and pending rows | Closed by tests | All source rows count. Tests cover 4095 existing rows, a final reservation and later refused onsets. |
| All terminal slots and full feedback | Closed by tests | 4096 accepted Gates complete without callback allocation or deallocation. A further onset remains unrecorded. |
| New pass, project reset or concurrent read | Closed by tests | Old identities return no row. A pending new Gate cannot inherit an earlier completion. |
| Stop before clock reset | Closed by tests | Gate duration uses the pre-reset boundary. Duplicate end commands cannot change it. |
| App publication, interrupted audio and erasure | Open | Tasks 2.2, 2.3 and 5.1 connect recovery, update raw rows and reuse erased capacity. |
| Shared physical inputs on one pad | Preserved | Existing manual per-pad release rules remain. Production input routing belongs to task 3.2. |

The new sound tests are in `crates/engine/tests/pattern_capture_gates.rs`.
Terminal identity tests are in `crates/engine/src/capture.rs`.
Allocator checks are in `crates/engine/tests/realtime_no_alloc.rs`.
Task 1.2 remains open until required checks pass and its authorized commit exists.

## Sound-preserving filter calculation boundary

The owner approved completion work, including removal of redundant filter calculations.
This boundary changes `crates/engine/src/filter.rs`, this design and `tasks.md`.
Unit tests stay in the filter module. No public action, saved field or sound rule changes.

A production-module probe shows that floating-point rounding can stop the filter glide short of its target.
After one and eleven seconds, its cutoff, damping and mix remain identical, but the old path recalculates coefficients every frame.
This observation identifies redundant work. It does not prove the cause of every callback overrun.
The probe, stack samples and unchanged benchmark control remain under `target/97-perf-resume-m8wdmpjh/`.

Keep the existing glide arithmetic, tolerances, integrators and bypass path.
Reuse coefficients only when cutoff, damping and sample-rate inputs have identical floating-point bits.
An absent cache must calculate them. A new voice must initialize its own cache.
Only the existing coefficient-update path can use this cache. Do not add updates where the old path skipped them.
Mix-only changes must continue their glide without recalculating identical coefficients.
Keep this fixed-size state inside each voice. Add no callback allocation, locking, logging or I/O.

Reject snapping the stalled glide to its target because that changes the sound.
Reject weaker workloads, percentile-only acceptance or a faster-core selection as a performance fix.
Compare every output sample and integrator state with the unchanged formula, bit for bit.
Cover fixed settings, changing targets, bypass, stalled transitions and sample-rate changes.
Run the original maximum-budget benchmark on CPU 12, the full required check, specification validation and release build.
A faster filter alone cannot close historical overruns, listening or physical-controller acceptance.

| Crossing | Required result | Verification status | Check |
| --- | --- | --- | --- |
| Stalled glide and repeated frames | Reuse coefficients without snapping state | Closed by tests | Bitwise old-formula comparison after one and eleven seconds |
| Type-only change and fixed coefficient inputs | Keep mix glide and integrators unchanged | Closed by tests | All filter types, including bypass transitions |
| Cutoff or damping changes again | Replace stale coefficients before processing sound | Closed by tests | Repeated targets and reversed changes |
| Sample-rate change during the old update path | Include the sample rate in the cache identity | Closed by tests | Different rates with unchanged cutoff and damping |
| New or default voice | Never inherit another voice's coefficients | Closed by tests | Fresh start, default initialization and restarted voice |
| Settled target or both sides bypassed | Preserve the old skipped-update behavior | Closed by tests | Sample-rate changes through both early paths |
| Callback maximum after optimization | Keep the 32-voice workload and 50 percent limit | Open, violations remain | Original benchmark and allocation tests |
| Stalled residual reaches the exact target | Separate sound decision | Requires decision outside this boundary | Preserve current residual state |

## Approved drive calculation boundary

The owner approved a separate drive optimization after the filter boundary.
This boundary changes `crates/engine/src/fx.rs`, this design and `tasks.md` only.
Tests stay in the effect module. Keep all existing workloads, limits, bindings and saved data.
The remaining callback maxima exceed the target. Stack samples identify drive processing as a candidate, not a proven overrun cause.

First test a local calculation prototype against the existing hyperbolic tangent formula.
Use a bounded rational approximation only if the complete drive output stays within 0.000001 per sample.
Keep the existing gain normalization, zero-amount bypass, mix and parameter ramps.
The local sweep rejects the eleven-term candidate because complete output exceeds the existing tolerance.
The thirteen-term continued fraction, written as a degree-13/12 rational function, passes the same sweep.
Evaluate it only below an absolute input of nine. Above that boundary, use signed unity.
Bound the rational result to minus one through one, as the original hyperbolic tangent does.
Check the chosen coefficients directly against the standard function, not only through another optimized engine.
Do not replace the effect's transfer curve, reduce voices, bypass effects or select a faster core as the fix.
Reject a lookup table, new dependency or platform-specific unsafe path unless another boundary explicitly permits it.
The candidate must use fixed arithmetic without allocation, locks, I/O or logging.

Cover positive and negative input, silence, subnormal values, saturation, amount limits and abrupt or smooth parameter changes.
Check values immediately beside each piecewise boundary. Compare values above the ordinary audio range too.
Check independent stereo output, finite samples, clipping, reference level, attack timing, silence and adjacent steps in the comparison render.
Keep the existing comparison WAV and its reference section. Add tests without weakening its assertions or tolerance.
Run debug and release tests, `task check`, CPU-12 benchmark, specification validation and the main-folder release build.
Keep timings, prototypes and traces local. No result from a microbenchmark closes the callback maximum requirement.
A successful optimization does not establish listening, physical-controller or live-terminal acceptance.

| Crossing | Required result | Verification status | Check |
| --- | --- | --- | --- |
| Zero amount and arbitrary sample | Preserve exact bypass | Closed by tests | Both signed zero values and finite sample extremes |
| Approximation boundary and opposite signs | Keep symmetry, continuity and the existing error limit | Closed by tests | Dense sweep and neighboring floating-point values |
| Quiet tails and subnormal input | Preserve silence and finite output | Closed by tests | Tiny values and decay through zero |
| Full amount and input beyond unity | Keep saturation and finite output | Closed by tests | Dense drive-output comparison, including overload |
| Repeated hits and amount ramps | No state leak, click or added delay | Closed by tests | Independent stereo reference and frame-by-frame comparison |
| New arithmetic in the callback | No allocation, deallocation or blocking | Closed by tests and source review | Existing production allocation tests and source review |
| Whole callback after the prototype | Retain workloads and maximum target | Open, violations remain | Original CPU-12 benchmark and full required check |

### Drive comparison listening

Use the local `renders/81-drive-comparison.wav`, 4.5 seconds at 48 kHz.
The reference keeps the previous left-channel signal. The right channel uses another tone for stereo coverage.
Drive stays fixed during the first second. The next second starts at zero drive and increases it.
The same change occurs in both reference and optimized sections.

1. Listen from zero to two seconds for the reference.
2. Check silence from two to 2.5 seconds.
3. Listen from 2.5 to 4.5 seconds for the optimized section.
4. Compare each channel with itself between the two sections.
5. Check matching level, tone, attacks and parameter changes.

The renderer and a separate WAV reader check both channels against the previous formula within 0.000001 per sample.
They also check finite values, clipping, level, one-frame attacks, adjacent steps and required silence.
The existing `renders/56-drive.wav`, `renders/56-two-slots.wav` and `renders/86d-portable-chops.wav` also receive numerically different samples within that tolerance.
The other 42 WAV files remain identical, including the final recording render.
Owner listening remains unverified. A comparison render does not establish the remaining callback maximum requirement.

## Approved temporary CPU-frequency experiment

The owner approved one diagnostic A/B/A experiment after failed callbacks received CPPC hardware observations.
This permission does not authorize a sound change, weaker workload, delivery action or permanent system setting.
The repository boundary contains this design and `tasks.md` only.
The local controller and tests stay in `target/97-cppc-tx6i0tsm/frequency_experiment.py`.
Keep previous observations and executable hashes separate from the new experiment evidence.

Use the same offline diagnostic executable, CPU 12, sampling stride, voices, effects and timing gates in all three phases.
Run the first and last phases with the original policy.
During the middle phase, request a minimum of 700000 kHz instead of 400000 kHz.
Only `/sys/devices/system/cpu/cpufreq/policy12/scaling_min_freq` can receive a privileged write.
Check that this policy contains CPU 12 alone before any write.
Keep the maximum, governor, driver, energy preference and all other policies unchanged.
A requested minimum is not proof of the hardware performance delivered during a callback.

The controller runs as the ordinary user. Only the fixed-path policy guard runs through `sudo`.
The guard must not start a benchmark, shell command or audio client.
Authenticate in the owner's terminal before the baseline phase. Never collect a password in chat or a log.
Without authentication, stop before the first benchmark or policy write.
Reject a changed initial minimum, shared policy or conflicting policy values.

The guard uses a 55-second watchdog, with five seconds reserved for normal cleanup within the approved 60-second window.
Restore the original minimum after the middle phase, timeout, parent-pipe closure, handled signal or failure after a write attempt.
Read back the restored value before starting the final phase.
If another writer selects a different minimum, stop instead of overwriting that value.
Keep failures and partial records. Do not restart the changed-policy phase automatically.
A forced guard kill, kernel failure or unavailable sysfs write can prevent restoration.
The owner must remain available for explicit recovery in those cases. A userspace watchdog cannot provide a hard real-time guarantee.

Test normal completion, partial write failure, timeout, parent loss, handled interruption, competing policy changes and failed restoration without real sysfs writes.
Also test authentication failure, a shared policy and benchmark execution under the ordinary user.
Retain all maximum violations, even when an older percentile-based exit returns zero.
A failed benchmark still requires restoration and the final baseline when restoration succeeds.
Reject a third speculative sound calculation as a substitute for this controlled diagnostic.
Even a successful frequency experiment cannot close historical maximum-budget failures or establish the cause of every CPU slowdown.
Listening, controller acceptance and the existing recording requirements remain unchanged.

### Completed owner experiment

The owner completed the approved A/B/A experiment from a terminal with private authentication.
All three phases used the same offline executable and ordinary-user identity.
The policy observations matched each requested minimum.
The normal guard path restored the original policy within the approved window.
A fresh policy observation also matches the original minimum, maximum, governor, driver and energy preference.

All three phases retained erasure maximum violations and nonzero benchmark exits.
The hardware observations still show reduced delivered performance during a failed callback with the higher minimum request.
This request did not prevent the observed reduction. It is not an accepted production solution.
The cause remains unknown. Keep the exact measurements, commands, traces and comparison analysis in the local evidence directory.

The normal host restoration path now has direct evidence. Failure recovery on the host remains untested.
Forced-kill, kernel-failure, policy-interference and observer limits remain unchanged.
Do not repeat the experiment without new owner permission.

## Approved read-only CPU-limit investigation

The owner approved a separate read-only diagnostic after the completed frequency experiment.
This boundary changes this design and `tasks.md` only. The controller, reader, tests and raw evidence stay under the existing local diagnostic directory.
Do not repeat the frequency experiment. Keep production sound, source, saved data, workload and the maximum requirement unchanged.

Only the fixed-path reader can run through `sudo`. The controller and benchmark must run as the ordinary user.
Authenticate privately before any workload. Refuse root execution of the controller.
The reader opens only `/dev/cpu/12/msr`, with read-only access and no symlink traversal.
Check the device identity, CPU model, required features and unchanged CPU-12 policy before measurements.
Reject an unexpected device or model instead of applying another model's register meanings.

Use this fixed register list:

- `0x34`: SMI counter, for firmware-interruption observations.
- `0x19a`: raw clock-modulation control.
- `0x19c` and `0x1b1`: core and package thermal status.
- `0x64f`: raw performance-limit reasons, without unverified model-specific bit names.
- `0x774`: the core hardware-performance request.

Use only eight-byte positional reads. Add no device writes, ioctls, counter resets, permission changes or module loading.
Keep unsupported reads and errors as unavailable data, never zero. Retain valid zero readings.
Interpret only fields supported by the installed Linux definitions. Preserve other bits as raw values.
Separate current status from sticky history. Kernel and firmware code can change status independently.

The reader cannot start a benchmark or shell, accept arbitrary paths or addresses, or transfer its device descriptor.
It sends bounded records through stdout. The ordinary-user controller owns the evidence files.
Use a finite reader deadline, parent-pipe closure and handled signals to end observation.
Stop on output backpressure instead of blocking indefinitely. A kernel failure can still prevent a hard deadline.
Retain partial records and nonzero exits. Do not restart the approved pair automatically.

Run one unchanged CPU-12 diagnostic control, then one identical diagnostic with the reader.
Keep the original executable, CPPC stride, effects, voices and timing gates in both phases.
Timestamp individual register reads. Report their duration and missing coverage.
Remote reads can interrupt CPU 12. A diagnostic success or failure does not by itself establish production acceptance.
Time associations use the original benchmark clock anchor and can be approximate. Do not claim exact attribution from a nearby snapshot alone.

Test device rejection, short reads, unsupported registers, valid zeros, root refusal, failed authentication and bounded output without host MSR access.
Also test parent loss, cancellation, deadlines, counter decreases and current-versus-sticky status interpretation.
The power, electrical, thermal, firmware and autonomous-control explanations remain hypotheses until the observations distinguish them.
No commit, publication, device playback or owner-project change belongs to this permission.

### Completed read-only pair

The owner completed the approved control and observed run after private terminal authentication.
Both phases used the original workload as the ordinary user and had no wall-time or CPU-time maximum violations.
All six registers were readable without errors. The reader ended normally after parent-pipe closure.
Saved policy observations and the final readback match the original policy.
Current power-limit status and package thermal status occurred during the observation interval.
The core thermal status bit stayed clear. The sampled SMI counter did not change.
These observations do not identify the cause of earlier failures: this pair captured no failing callback.
They also do not measure the reader's CPU-12 interruption cost or prove that the reader missed no short event.
The run and independent audit stay under `target/97-cppc-tx6i0tsm/msr-a58a5173ce98469cb53d269c62627336/`.
The approved pair is complete. Do not repeat it without new owner permission.
Keep the original maximum requirement, historical failures and owner acceptance open.

## Owner trial feedback and deferred editor direction

The owner reported that recording looks acceptable, but erasure is unclear without a visible pattern editor.
This is preliminary recording feedback, not acceptance of the full change or its timing limits.
A separate review of existing screens and code supported that concern. It did not execute a new input or live-device trial.

The owner accepted this visual direction for the next editor:

- Show the selected pad's recorded events in a strip below the large waveform.
- Distinguish occupied steps from empty steps and group cells by musical beat.
- Show the current bar, loop length, view division and playback position.
- Keep the selected edit position separate from the moving playback position.
- Use an explicit `STEP EDIT` mode. Keep each physical pad associated with its original sound.
- At the selected step, light every pad that has an onset there. Pressing a lit pad removes its onset there, not its other onsets.
- Keep ordinary pad playing outside that edit mode.

The example used 16 cells at 1/16. It does not limit playback or the future editor to that division.
Existing playback supports Off, 1/4, 1/8, 1/16, 1/32, 1/64, 1/8T, 1/16T and 1/32T.
The next design must account for these divisions and for recorded timing between grid lines.
A view change must not change raw timing or playback quantize. Preserve velocities and Gate lengths.
Do not silently merge multiple events into one binary cell or remove unrelated events from that cell.

This section records follow-up intent only. It does not activate 98 or authorize a step grid in 97.
Keep the current saved format and transport behavior. Create the next change's requirements and tests before code.

## Migration plan

Reuse format 5 without new recording fields. Add no owner-project cleanup or migration task.
Implement and test in atomic steps. Keep normal playback regressions and original benchmark workloads.
Before owner handoff, run the normal main-folder `task build` after any other worktree operations.
Never share a Cargo target directory between worktrees.
Rollback uses an earlier binary with the same format support and a complete project backup, not a lower format number.

## Open questions

The owner agreed on recording, two-bar creation, count-in, overdub, pass Undo and staged visuals.
The owner also approved the first-repeat and eligible-collision rules in "Preserve the previous rhythm during overdub".
The owner approved the Pitch Gate feedback clarification without changing its sound rules.
The owner approved F1 wrapping, Alt+PageUp/PageDown and two learnable page commands, without factory controller assignments.
Ordinary bank and pad controls remain. Accepted recording feedback stays visible while F1 is open.
The following defaults are design choices, not claims of separate owner approval.

- Does manual save need to continue an active overdub? Default: finish the pass, then save while playback continues.
- Should Ctrl+Z retain grid changes made during the pass? Default: restore both raw rows and those pattern-setting changes.
- Is one loop sufficient for erase targeting without releases? Default: one loop per pad press, with a visible fallback hint.
- Are Alt+N, Alt+E and Alt+Z comfortable on the owner's keyboard? Default: use these unoccupied chords and effective-key help.
- Should recording on Chop leave slice audition available on pad keys? Default: real assigned pads during pattern work, preview through existing controls.
- Are `PG A1` and `PG n/a` clear enough? Default: use these six-cell-or-shorter labels with the explicit help texts above.
  The owner approved the three-state distinction, not these exact compact labels.

These choices are fully specified for this draft. A different choice requires a specification edit before code.

The filter glide can stop short of its target because floating-point rounding prevents another step.
Should a later change force the exact target, with a possible numerical sound change?
Default for this boundary: preserve that residual state and avoid redundant coefficient calculations.

Can this CPU minimum request prevent the observed hardware-performance reduction?
The completed experiment retained the observed reduction and maximum failures with the higher request.
Default: reject this request as a production solution. Keep the cause open and preserve the original policy.
The watchdog cannot recover from its own forced kill. Another live policy experiment requires new permission.

Which hardware limit or firmware event accompanies the remaining CPU-performance reductions?
The completed read-only pair captured limit-status signals but no failing callback, so this question remains open.
Default: preserve this evidence and keep uncertain time associations explicit. Do not repeat the approved pair automatically.
Do not select another frequency request or sound change without a new boundary.

### Deferred editor questions

How should the next editor page dense grids and show off-grid events or several events in one cell?
Proposed default: retain readable cells, show the page span and event multiplicity, and preserve every event when the view changes.
The exact event-selection rule needs a specification before removal or insertion code.
Should an empty pad insert a new event at the edit cursor?
Default: do not treat insertion as separately approved by the accepted removal example.
Does the owner's request for different beats include meters other than 4/4?
Default: existing transport remains 4/4. Other meters need a separate explicit scope decision.
Keyboard and controller paths, event-duration indicators and the final page layout also remain design questions.

### Deferred audio-input level work

The owner reports low levels when recording the audio input and requests a later solution for very quiet recordings.
Current Chop normalization changes pad gain and cannot exceed +6 dB. That limit can prevent the -1 dB peak target.
No input measurement exists. The cause remains unknown.
Future work must provide a way to bring very quiet recorded material to a usable level beyond this pad-gain limit.
Input gain, post-record sample gain and automatic normalization remain separate design options, not approved implementations.
Increasing gain also increases recorded noise. Peak normalization does not equalize perceived loudness or provide compression.
Open question: where should level correction occur, and should the owner apply it explicitly?
Default for 97: keep current audio-input gain and recorded samples unchanged. Prepare a separate specification before implementation.
