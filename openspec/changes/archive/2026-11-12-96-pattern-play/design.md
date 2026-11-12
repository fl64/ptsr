# Design: one looped pattern

## Context

See [the proposal](proposal.md#why) for the owner result.
The baseline is merged master `ac7596925fc76949d69f0430302d1e60dd61bd0a`.

Verified sources:

- `crates/engine/src/transport.rs`: audio-frame clock, 960 ticks per quarter note, coherent epoch-tagged publication and nearest-frame metronome beats.
- `crates/app/src/session.rs`: format 4 stores tempo and metronome preference. It has no pattern fields.
- `crates/engine/src/lib.rs`: manual hits cut the previous same-pad voice and its mute group. Releases currently identify a pad, not a note.
- The same engine file mixes pad voices before its frame-by-frame metronome pass. Scheduling future hits before mixing would cut earlier voices too soon.
- `crates/engine/src/rt.rs`: project epochs, restore barriers and retirement capacity guard queued commands.
- `crates/app/src/pads.rs`: swap, copy, reset and undo snapshots hold pad settings. Source identity distinguishes `all`, linked `slice N` and `pad`.
- `crates/app/tests/scenarios.rs`: active scenarios need real test declarations. A planning draft alone cannot pass that audit.
- `config/keymap.toml`: Alt+G has no shipped role. Existing page knobs already have three parameters each.

These facts describe the baseline, not a working sequencer. All behavior below is proposed for 96.
The owner approved the proposal scope and the swap and grid-boundary choices before this draft.

## Goals / Non-Goals

**Goals:** one clock, exact hit and release timing, bounded callback work, safe project isolation and reversible pad edits.

**Non-goals:** no generic event framework, recording queue, future slot system or new parameter page.
Do not add future recording fields such as `skip_lap`. No dependency or audio-device access is necessary for offline checks.

## Decisions

### Saved format

Use session format 5 for both projects and scratch. Keep all existing fields.
The example below supplies top-level fields before any existing TOML table. Do not paste a second `version` or `bpm` field.

```toml
version = 5
bpm = 120.0
metronome = false
pattern = "A1"

[[patterns]]
slot = "A1"
bars = 2
quantize = "1/16"
strength = 100
# [raw tick, absolute pad, velocity, gate length in ticks]
hits = [[0, 0, 100, 0], [1920, 1, 110, 0], [3840, 0, 100, 0], [5760, 1, 110, 0]]
```

This is the complete pattern syntax, not a complete project with sample assignments.
The file permits zero patterns or one entry named `A1`. `pattern` is absent without that entry and is `A1` with it.
An empty `hits` array is valid. There is no second slot or slot-selection control.

| Field | Contract |
| --- | --- |
| `bars` | Integer 1–64. A bar has 3840 ticks. |
| `quantize` | `off`, `1/4`, `1/8`, `1/16`, `1/32`, `1/64`, `1/8T`, `1/16T`, `1/32T`. Default `1/16`. |
| `strength` | Integer 0–100, multiples of 10. Default 100. |
| `hits` | At most 4096 rows, each with exactly four integers. |
| Raw tick | 0–245759, including hits outside the current shortened loop. |
| Absolute pad | 0–127. Pad 0 is bank 1 key `1`. Pad 16 is bank 2 key `1`. |
| Velocity | 1–127. Playback gain uses velocity / 127. |
| Gate length | 0–245760 ticks. One-shot mode ignores it. Zero-length gate notes are silent. |
| `pitch_gate` | Boolean on each saved pad. Absent means true. It acts only on qualifying pattern notes. |

Reject invalid fields, duplicate slots, mismatched selection and unsupported extra slots before replacing current work.
Do not clamp, discard or rewrite invalid raw hits. Keep the existing load-refusal and write-protection routes.

Alternative: one pattern object without a slot label. Rejected because the planned owner-facing identity is `A1`.
Do not generalize its storage into sixteen functioning slots before change 99.

### Raw hits and playback quantize

Keep the file's raw hits and their order. Build an immutable playback list outside the callback.
First exclude raw ticks at or beyond `bars * 3840`. Retain them in saved data.
Grid spacings are 960, 480, 240, 120, 60, 320, 160 and 80 ticks in the listed order, excluding Off.

Choose the nearest grid line. An exact midpoint chooses the later line.
Move toward it by `strength` percent. Round the result to the nearest integer tick, with an exact half choosing the later tick.
Off preserves the raw tick regardless of strength. Wrap only an included hit whose quantized tick equals the loop length.
Sort by tick, then absolute pad. Merge collisions on the same pad and tick.
Keep the higher velocity. For equal velocity, keep the longer gate length. An exact tie keeps the earlier raw row.
Do not merge the raw rows or remove hidden hits.

This preserves future recording timing without implementing a recorder. Destructive quantize would lose that timing.
Changing loop length or quantize currently requires closing ptsr, editing the file and reopening it.
Runtime replacement remains an engine contract for pad swaps and deterministic tests, not an external file watcher.

### Scheduling through the existing clock

Use `Arc<Pattern>` with checked immutable event storage. Send `SetPattern` through the existing epoch-tagged project route.
Do not send sequenced hits through the UI. A stalled redraw must not stall sound.

Let `t` be accepted transport ticks at the next output frame and `d` the ticks per frame at the current tempo.
A due event satisfies `event_tick <= t + d/2`, with the transport's rounding tolerance.
Track the next event on the unwrapped transport timeline. Consume it once, including events rounded across a callback boundary.
A tick at the loop end belongs to the next lap, never both laps.
Empty blocks change no scheduler cursor, release deadline or clock position.
Tempo changes affect future tick advancement, not stored musical deadlines.

Within an output frame, process due old releases before new pattern hits. Order simultaneous hits by absolute pad.
Manual commands retain accepted block-start order and act before that frame's scheduled pattern events.
Same-pad retrigger and mute groups retain their existing cuts. At coincident manual and pattern onsets, the later pattern trigger wins.

Mix only up to the next event or release boundary, then apply that event.
Retain callback-relative gain, drive and send interpolation across those spans.
Do not restart a glide, effect, delay line or metronome pass for every span.
The exact helper layout is an implementation detail. Tests must catch early cuts and changed block-boundary smoothing.

Alternative: call the current trigger with every future offset before mixing. Rejected because same-pad and group cuts would act too early.
Alternative: UI timer. Rejected because its timing depends on redraws and stalls.

### Note ownership and release deadlines

Give each sequenced voice a note identity distinct from manual voices and later sequenced hits.
Keep fixed-capacity pending releases with absolute musical deadlines and their release kind.
A gate note uses its saved length and the existing pad release envelope.
A one-shot ignores gate length. Stop releases sequenced gate notes but lets ordinary one-shot tails and shared effects finish.

A Pitch Gate cut uses the existing 5 ms cut path, not the pad's potentially long gate release.
An earlier ordinary release starts its envelope. The later Pitch Gate deadline still caps that fading voice with the short cut.
If they coincide, use the short Pitch Gate cut. Starting it must preserve the current fade gain.
Stop or pattern replacement also cuts a still-active Pitch Gate note through that short path.
Do not release manual voices, previews or unrelated one-shot tails.
A manual key release must not release a newer sequenced note merely because it uses the same pad.
Keep existing manual-to-manual release behavior unchanged.

Remove stale deadlines when voices end, retrigger, are stolen or retire.
A source edit alone does not remove a sounding note's deadline. Replacing that voice invalidates its old identity.
Use 64 pending entries. If a full set remains, apply the earliest due release before inserting another entry.
Break deadline ties by note identity. Never resolve overflow through a pad-wide release.
Identity reuse must invalidate old entries before a slot can hold another note.

At a loop seam, apply due releases before tick-zero hits. A saved gate length can cross the seam.
Stopping resets the timeline and discards pending deadlines after applying their required releases.
A later start cannot inherit a release from an earlier run.

### Slice Pitch Gate

Use one saved boolean per pad, default true. Eligibility at note onset requires a linked `slice N` source and pitch above zero.
Whole files, detached `pad` pieces, zero or negative pitch, manual hits and Chop previews receive no Pitch Gate deadline.
Changing pitch, source or the switch affects eligibility of future notes. It does not move a deadline already attached to a sounding note.
Natural source end always wins over a later deadline. Never stretch, repeat or read beyond that end.

The deadline is the first selected grid line strictly after the compiled onset, including an empty step.
Off uses 240 ticks (1/16), even with strength 0. Triplets use their exact integer spacings.
The loop end is a boundary. Swing is absent and cannot silently change this grid.
Start the existing short cut at the deadline within one output frame. Its fade can continue into the next step.
Do not promise full silence exactly at the boundary or extend a short source to fill that step.

Alternative: wait for the next recorded hit. Rejected by the owner's grid-boundary choice because empty steps must cut the tail too.
Alternative: cut every slice, regardless of pitch or source. Rejected because this feature is the planned pitched-up linked-slice gate.

### Replacement, swaps and project changes

A same-project replacement acts at one callback boundary. Preserve transport ticks and metronome phase.
Release the old pattern's gate and Pitch Gate notes, then replace the playback list.
Choose the first new event whose nearest output frame is not before the boundary. Do not replay past events.
Removing the pattern leaves the clock running without further sequenced hits.

A pad swap changes both pad states and all raw hit addresses in one logical transaction.
Include hidden raw hits. Keep positions, velocities, lengths and loop settings unchanged.
Deliver engine pad swap and replacement pattern in one accepted transaction, not separate commands that can split across callbacks.
Undo restores both pad snapshots and hit addresses in one step. Cancellation, self-target and loading refusal change neither.
Unlike general replacement, the swap transaction preserves voices and deadlines on unrelated pads.
Affected voices keep the existing swap fade. Their old deadlines cannot target replacement notes.
File replacement, clear and copy keep event addresses. A copy duplicates pad settings, not hits.

A project transition clears pattern state and note ownership under the existing fade and restore barriers.
A queued old pattern, swap or release cannot reach the target project.
A refused target read or required source save leaves the source rhythm running.
Selecting the current folder must not reload it or stop its pattern.

### Real-time ownership and pressure

Extend garbage retirement to carry either sample or pattern references. Drain and free them outside the callback.
Check capacity before accepting replacement, swap, restore and obsolete-command disposal.
If retirement cannot proceed, retain ownership and defer the whole transaction without waiting or partial application.
Keep the old pattern playing during a deferred same-project replacement.
A project switch still silences the old project before target readiness, even if retirement is delayed.

Automatic triggers cannot wait for a full retirement queue or accumulate late hits.
If bounded voice retirement cannot accept a new automatic hit, omit that hit and increment a published dropped-hit counter.
Keep the clock and subsequent events on time. Show a warning when the UI resumes.
Never free the last sample or pattern reference in the callback to make a hit fit.
A normal dense workload must produce zero dropped hits.

Do not weaken current allocation/deallocation tests, scenario audit, benchmark workload or exit conditions.
Add a dense 172 BPM pattern measurement with 32 voices and active existing effects at 128 frames / 48 kHz.
Report wall and thread CPU maxima, p99 and exceedance counts separately.
The existing 50% maximum target remains distinct from mandatory p99 exit checks.

### Controls and visible state

Use `pad.pitch_gate_toggle`, shipped as Alt+G on Pads with edit open or closed.
Set every scope pad to the inverse of the selected pad's value. Make one undo step for the change.
No change means no undo or save activity. Reset sets true. Swap and copy carry the value with the pad snapshot.
Clear keeps this parameter like the other retained pad parameters.

Offer the same button action in F12 MIDI under PADS. F10 selects it by its effective key without executing it.
Existing confirmation, cancellation, clear, press/release and saved-binding rules apply.
No knob, new page or factory control is necessary. Preserve TONE, ENV, FILTER, SOURCE and FX and all existing knob targets.

Show `PG on` or `PG off` beside the selected pad's worded mode, with an inactive style for an ineligible source or pitch.
F1 explains the eligibility and shows effective keys and learned controls. An applied toggle uses the existing value popup.
Do not add a new pad-input mode or yellow grid outline for this boolean.

Extend the coherent transport snapshot with accepted pattern presence and length.
Derive the loop bar/beat from that same snapshot, not the UI's current proposed pattern.
Show `A1` and its loop-relative position while retaining the existing transport bar/beat where it differs.
At narrow widths, keep the existing transport and safety priorities. The compact slot is `A1`.
Sequenced voices count as actual pad playback for PLAY, flashes and ending the startup splash.
They do not select another pad, mark pads, simulate held input or change KB/controller identity.

### Owner trial and independent workflow evidence

Deliver a local trial after the offline checks. Use a copy of an owner project with slices, never its original files.
Keep the copied audio, settings, project, scratch data and logs under ignored `target/96-owner-trial/`.
Include only copied recording files in the trial. Remove references to external downloads and unrelated samples.
Preserve the selected slices and their older chop states. Supply a ready two-bar A1 rhythm and an owner-only start script.
The owner must not enter TOML for the first trial. Recording and the step editor remain outside 96.
Start stopped, with a conservative trial master level. Document each intentional trial-only setting change.
Do not copy saved owner MIDI assignments or claim an SMC-PAD factory layout.

The owner requested one-time cleanup of existing project files instead of more migration work.
Do not add conversion paths for that cleanup. Preserve the existing compatible readers.
Back up each closed older project before updating its header to format 5 and making its existing transport defaults explicit.
Keep all pad, chop, path, group, name and effect data unchanged. Do not change the project open in the owner's running sampler.
This maintenance does not add A1 rhythms to the original projects or move their audio.
Keep backups and identifying file records local, outside Git.

A separate reviewer follows the documented closed-program entry and keyboard workflow before delivery.
Its terminal process must deny audio and MIDI device access, not only select a nonexistent sound server.
Use kernel file-access restrictions for device paths and writes outside the review folder when container namespaces are unavailable.
Keep the existing JACK isolation variables and 1024-frame test setting. Refuse to start if isolation checks fail.
A device-free terminal cannot advance the audio clock. Use real offline callback tests for that part of the workflow.
Report terminal observations, offline sound evidence, physical controls and listening as distinct results.

## Risks / Trade-offs

- [Manual TOML entry] → Document one closed-program workflow. Recording and editing remain separate changes.
- [Shared voice identity] → Test both orderings of manual and sequenced hits, stale releases, slot reuse and note stealing.
- [Boundary rounding] → Compare 1-frame, 128-frame, 1024-frame and irregular blocks at 40, 120, 172 and 300 BPM.
- [Callback span changes] → Compare manual and sequenced sound, smoothing, effects and metronome timing with permitted generated fixtures.
- [Resource pressure] → Defer atomic replacements, never delay hits, count omitted hits and test recovery after queues drain.
- [Older binaries] → Require backups. Do not offer a lossy automatic downgrade.
- [Controller prerequisites] → Use current learnable actions. Physical SMC-PAD reach and unfinished 88–93a remain unverified.
- [Historical callback maxima] → Report new results without calling deferred investigations fixed.

### Boundary review

Status describes the design contract, not test completion.

| Case | Expected result | Status | Planned evidence |
| --- | --- | --- | --- |
| Old release, manual retrigger, old deadline | New manual voice survives | Closed in design | Engine render plus raw MIDI and keyboard routes |
| Manual press, sequenced retrigger, manual release | New sequenced note keeps its own deadline | Closed in design | Reversed-order ownership tests |
| Last frame, loop wrap, tick-zero hit | One hit, old release first | Closed in design | Block-size timing renders |
| Shorten, quantize, grow | Hidden raw hits do not wrap into the beginning | Closed in design | Compile and session round trip |
| Swap, full command queue, undo | Pad sounds and event addresses never split | Closed in design | Production routes and callback observation |
| Old project command after target restore | No target hit, release or pattern replacement | Closed in design | Epoch and saturated-retirement tests |
| Save refusal during a playing project switch | Source rhythm and pending work remain | Closed in design | Injected write refusal without live devices |
| Full garbage queue during automatic playback | No free, no wait, no late burst, visible counter | Closed in design | Allocation guard and recovery test |
| Stop, empty blocks, restart | No old deadline and exactly one tick-zero hit | Closed in design | Engine transport regressions |
| Hardware reach and subjective clicks | Do not infer acceptance from offline tests | Open | Separate owner checks |

Axes checked: transport state, source kind, note origin, block boundaries, command ordering, project lifetime and queue pressure.
Swing, multiple slots, recording passes and controller holds are outside this design.
The largest remaining uncertainty is callback cost with dense scheduling. Measure it before accepting the implementation.

## Migration Plan

1. Read and check formats 1–4 through their existing migration paths.
2. Restore no pattern for formats 1–4 without pattern data. Refuse a pattern selection or entry under an older format number. Do not discard it. Preserve tempo, metronome, pads, groups, sources, names, paths, chops and effects.
3. Default absent Pitch Gate state to true. With no pattern and for manual play, this has no audible effect.
4. Save format 5 only through the existing safe save and retry route. Preserve read protection and missing-folder refusal.
5. Reopen format 5 stopped at tick zero. Do not save running state, voice identities or pending releases.
6. For rollback to an older binary, restore the backup. Do not edit only the format number to hide new fields.

## Open questions

The owner approved swaps following sounds and the next-grid boundary, with Off using 1/16.

Independent review found an unresolved Chop interaction. Should a running pattern continue while Chop is open, or should Chop suppress future automatic hits for solo preview?
The local trial retains the implemented default: entry fades the currently sounding pads, but transport and future pattern hits continue.
Space remains Chop preview. Stop transport on Pads before entering Chop for solo listening.
This default is an observed trial behavior, not owner acceptance or a new suppression policy.
The earlier Chop requirement promises silence on entry so the owner can hear slices alone.
Resolve its interaction with continuing automatic hits in a specification delta before archive.
Do not silently stop the clock or suppress notes.
The exact generated kick and snare timbres can be chosen during test creation. Default: short synthesized tones with distinct frequencies.
That fixture choice must not change timing, source eligibility, audio-origin rules or acceptance thresholds.
