# Plan: transport, patterns, recording and the sequencer (changes 95–101)

Read `README.md` first. Key choices follow `KEYMAP-CLEANUP.md` (83a). This track needs
the input work (84–90), the SMC profile and panel
(91–92), the reach checks (93) and comfort (93a, including redo). Pad lights (94) are optional.
The separate callback budget change before 95 is a prerequisite, not optional reporting work.
Each change states its keyboard keys and its SMC-PAD path. The reach test of 93 covers them.
The design sketches below are proposals, not existing Rust interfaces or saved file formats.

The additions below support fast breaks, fills and detailed step control inside 95–100.
[SAMPLE-WORKFLOWS.md](SAMPLE-WORKFLOWS.md) continues with sample tempo and sound work in 102–108.

## 1. Where we start

- There is no transport: no tempo, no play / stop, no clock. `Space` is `metronome.click`, a
  test click.
- The engine is ready for exact timing: `Engine::handle(offset, cmd)` takes an offset inside the
  block, and `Voice.delay` starts a voice in the middle of a block. The offline render uses this.
  In real time, `Realtime::process` calls `handle(0, ...)`: a command acts at the block start.
- A pad hit from the keyboard or a controller reaches the engine in the next block. The error
  is up to one buffer (128 frames at 48 kHz ≈ 2.7 ms; 1024 frames ≈ 21 ms).
- Samples go to the engine as `Arc<Sample>` through `rtrb`, and back through the garbage queue
  (`Producer<Arc<Sample>>`). Project switches use `ProjectCommand` with an epoch.

## 2. Design decisions (shared by all changes here)

### 2.1 The sequencer lives in the audio thread

The clock, the pattern play and the metronome are in `crates/engine`. The UI never times hits.
A frozen UI does not stop the beat. Files: `engine/src/transport.rs`, `engine/src/sequencer.rs`.

### 2.2 Time is ticks, 960 per quarter note

```
frames_per_tick = sample_rate * 60 / (bpm * 960)      // f64
```

The transport keeps `tick: f64` and adds `block_len / frames_per_tick` per block. An event at
tick `t` inside the block starts at `offset = round((t - tick_at_block_start) * frames_per_tick)`.
A tempo change acts at the next block start; the tick position does not jump. 4/4 only.
Tempo 40–300 BPM.

### 2.3 A pattern is raw hits plus settings; the engine gets a compiled list

App side (`crates/app/src/pattern.rs`), saved in the project:

```rust
pub struct PatternData {
    pub bars: u8,              // 1..=64
    pub quantize: Grid,        // Off, 1/4, 1/8, 1/16, 1/32, 1/8T, 1/16T, 1/32T
    pub strength: u8,          // 0..=100, steps of 10
    pub hits: Vec<Hit>,        // as played, not quantized; max 4096
}
pub struct Hit { pub tick: u32, pub pad: u8, pub velocity: u8, pub len: u32 }
```

`pad` is absolute 0..127 (owner question in README). `len` is the gate length in ticks; a
one-shot pad ignores it.

Engine side: `Arc<Pattern>`, immutable, built by `compile(&PatternData) -> Pattern`:

```rust
pub struct Pattern {
    pub len_ticks: u32,
    pub events: Box<[Event]>,  // sorted by tick
}
pub struct Event { pub tick: u32, pub pad: u16, pub velocity: f32, pub len: u32, pub skip_lap: u32 }
```

`compile` first leaves out raw hits at or beyond the current pattern length. Those hits
stay saved and return when the length grows. It then applies quantize to the visible hits
(`tick' = tick + (grid_tick - tick) * strength / 100`, where `grid_tick` is the nearest grid
line). Only a quantized visible hit that crosses the loop end wraps to the start. Merge two
hits of one pad on one tick (the louder one stays). Quantize is **non-destructive**: change
it any time, the raw hits stay. Swing (later) goes into `compile` too. Change 96 needs tests
for shortening, growing and a quantized hit at the loop boundary.

### 2.4 Changing a pattern while it plays

The UI builds a new `Arc<Pattern>` and sends `Command::SetPattern`. The engine swaps it at a
block start, keeps the tick position, and finds its place in the new list by binary search. The
old `Arc` goes to the garbage queue. Change the garbage queue to carry an enum:

```rust
pub enum Garbage { Sample(Arc<Sample>), Pattern(Arc<Pattern>) }
```

### 2.5 Recording goes engine → UI → engine

While recording, the engine handles a `PadPress` / `PadRelease` (from any input) and writes
`Recorded { tick, pad, velocity, press, lap }` into an `rtrb` queue to the UI (capacity 512; an
overflow increments a counter in `Stats`, the UI shows a warning). The UI adds the hit to
`PatternData`, compiles, and sends the new pattern. The hit sounded at once when it was played.

The double hit trap: a hit played just before a grid line is quantized forward, to a tick the
playhead has not reached. The new pattern would play it again in the same lap. Rule: the UI sets
`skip_lap` of a fresh hit to the lap the engine reported; the engine does not play an event whose
`skip_lap` equals the current lap. Lap 0 is never used as "skip nothing"; use `u32::MAX`.

### 2.6 The transport position goes to the UI by atomics

`Stats` (or a new `TransportState`) holds `tick: AtomicU64` (tick × 16 for sub-tick), `playing:
AtomicBool`, `recording: AtomicBool`, `lap: AtomicU32`. The UI draws bar.beat and the playhead
from them.

### 2.7 What a pad hit means in the new modes

Add to `Mode` (change 85 made it): `Step` (a pad hit toggles a step, routed to the UI, no fast
sound), `Erase` (a held pad erases its hits under the playhead, no sound), `SlotPick` (a pad hit
picks a pattern slot, no sound). `Record` is not a mode: pads sound and the engine records them.

## 3. The changes

### Before 95 — `94a-callback-budget`

Goal: resolve the callback peaks above 50 % and slow-core CPU spikes before sequencer work.
Dense rolls at 172 BPM add callback work. A report alone does not close this gate.
This is a separate small change. It does not depend on optional pad lights (94).

Spec deltas: `audio-engine`, for any explicit performance correction and its regression scenarios.
Keep the existing 50 % target and real-time rules.

Tasks:

1. Reproduce the earlier peaks with wall and thread CPU measurements.
   Record the workload, core classes and repeated runs. Define the supported test conditions in the design.
2. Fix the measured cause in a small change. Do not hide a behavior change inside a refactor.
   Do not replace the fix with an unapproved system scheduling or affinity change.
3. Check 32 voices, 128 frames / 48 kHz and project-switch phases against the 50 % target.
   Record maxima and exceedance counts, not only p99 or the command exit status.
4. Add dense-pattern and roll workloads in 96 and 100, once those paths exist.
   Keep the same target. Reopen the gate if a new path exceeds it.

Tests: the original slow-core and peak cases, repeated callback measurements, no allocation
and existing sound regressions. Do not claim closure from one favorable run.

How to verify: review the repeated measurements and the unresolved limits before starting 95.
If a sound path changes, render it and give the owner a listening comparison.

### Change 95 — `95-transport`

Goal: tempo, play / stop, a metronome on the beats. This is row 7 of `docs/BACKLOG.md`, without
the BPM detection of samples.

Spec deltas: new capability `transport`; `keyboard` (Space); `midi-learn` (GLOBAL: play / stop,
metronome, tempo knob); `projects` (the tempo is saved with the project).

Engine:

1. `Transport` with `playing`, `bpm`, `tick`, `metronome: bool`, `count_in: Option<u32>`.
2. Commands: `SetTempo { bpm: f32 }`, `TransportPlay`, `TransportStop`, `SetMetronome { on }`,
   `CountIn { bars }` (used by 97).
3. The metronome uses the existing click voice: velocity 1.0 on beat 1 of a bar, 0.6 on the other
   beats, exact offsets.
4. `TransportStop`: the clock stops and goes back to tick 0. One-shot tails ring on. From
   change 96, sequenced gate notes get their release. Panic is a separate action that also
   fades voices, preview and FX tails.
5. Half-time is a transport setting for pattern playback at half speed.
   Project BPM, the bar clock and the metronome stay unchanged.
   Define phase changes and gate-release rules in 95. Apply the playback factor in 96.

App:

1. `bpm` in the project (session version 4 with `bpm = 120.0` when absent; this is the migration
   start for 96 too).
2. `Space` on Pads, FX and Settings: `transport.play_stop`. Chop and Files keep their own Space.
   `metronome.click` stays an action for learn but has no default key.
3. Tempo on the keyboard: `Ctrl+Up` / `Ctrl+Down`, 1 BPM; with Shift 0.1 BPM. Check them
   against `config/keymap.toml` and against the terminals of the README (Ghostty took
   `Ctrl+0` before, see backlog row 22). If a terminal takes them, write it in the design and
   pick other right-half keys.
4. `metronome.toggle`: pick a right-half keyboard key in the design and check it for clashes.
5. The title line shows `120.0 BPM  ▶ 3.2` while playing.
6. Add half-time on/off with keyboard and controller paths. Save its state with the project.
   Show `HALF` next to the transport state. The default is normal speed.

SMC-PAD: PLAY tap on Pads is `transport.play_stop`; PLAY hold is `metronome.toggle` (the tap /
hold of 89; the tap then acts on release, write it in the spec); bank 2 knob 7 is the tempo. The
reach test of 93 must pass with the new actions.

Visual: the transport segment of the status line and the `▶ 2.3` badge (`VISUAL.md` §2.6, §2.1);
the value pop-up for the tempo.

Tests:

1. Render: 8 bars at 120 BPM with the metronome, blocks of 64, 128 and 1000 frames: every click
   at its exact frame (±1), accents on beat 1.
2. Render: tempo 120 → 90 in the middle: no jump in position, the clicks follow the new tempo.
3. Stop and play: the first click at frame 0 of the play.
4. `realtime_no_alloc` covers the transport.
5. Input: Space starts and stops. The SMC-PAD PLAY does the same (parity harness).
6. Render metronome accents at 170, 172 and 180 BPM. Check the first beat of each bar.
7. Half-time selection and restart preserve project BPM and metronome timing.
   Pattern duration and release tests follow in 96.

Render for the owner: `renders/95-transport.wav`.

### Change 96 — `96-pattern-play`

Goal: one pattern plays in a loop with the transport. No recording and no screen yet; the owner
writes a pattern into `project.toml` by hand to check it.

Spec deltas: new capability `patterns`; `projects` (patterns saved; version 4); `pad-assignment`
(swap moves the hits).

Engine:

1. `Pattern`, `Event`, `SetPattern { pattern: Option<Arc<Pattern>> }` as a project command
   (epoch rules of `ProjectCommand`).
2. In `process`, per block: the tick range `[t0, t1)`, with the wrap at `len_ticks`; each event
   in the range → the internal pad trigger with its offset. Use one internal function for pad
   triggers from `handle` and from the sequencer, so mute groups, pad mode, envelope, filter and
   sends act the same.
3. Gate notes: a fixed array of 64 pending releases `(pad, release_tick)`; when full, release the
   oldest at once. Stop sends all pending releases.
4. A hit on a pad with no sound is silent; no error.
5. The `Garbage` enum (§2.4).

App:

1. `PatternData`, `compile`, the session format:

   ```toml
   bpm = 120.0
   pattern = "A1"          # the current slot

   [[patterns]]
   slot = "A1"
   bars = 2
   quantize = "1/16"
   strength = 100
   # [tick, pad, velocity, gate length in ticks]
   hits = [[0, 0, 100, 0], [1920, 1, 110, 0], [3840, 0, 100, 0]]
   ```

2. Migration 3 → 4: no patterns, bpm 120.
3. `Ctrl+X` swap: hits of the two pads trade pads in every pattern (if the owner confirmed).
4. The title line shows the slot, for example `A1 ▶ 1.3`.

Visual: the slot in the `▶` badge and the transport segment (`▶ 120.0 · A1 2.3`).

Tests:

1. Render: a 2-bar pattern at 120 BPM: each hit at its frame ±1, over 4 loops.
2. Render: a gate pad hit with `len`: the release at its frame.
3. Render: pattern replaced while playing (one hit added ahead of the playhead): no hit missed
   or doubled around the swap.
4. Render: mute group pads in a pattern cut each other.
5. Session round trip and the migration from version 3.
6. Half-time doubles a pattern's duration without changing project BPM or metronome timing.
   Check gate releases, loop boundaries and a rate change under the rules from 95.
7. Benchmark dense patterns at 172 BPM against the callback gate before 95.

Render for the owner: `renders/96-pattern-play.wav`.

How to verify (owner): paste the TOML above into the project, load the kick on pad 1 (bank 1,
`1`) and the snare on pad 2 (`2`), start with Space.

### Change 97 — `97-pattern-record`

Goal: live recording into the current pattern with overdub, quantize, erase, undo of a pass,
and a count-in.

Spec deltas: `patterns` (record, overdub, quantize, erase, undo, count-in, limits); `keyboard`;
`midi-learn`; `command-panel` (Pattern cells start in 98; here RECORD is enough).

Behavior:

1. Record toggle while stopped: one bar of count-in with the click, then play and record from
   tick 0. While playing: record from now.
2. Overdub always: new hits add to the old.
3. A pass is from record on to record off; one pass is one undo step (`Ctrl+Z`, panel Undo).
4. The quantize and strength are settings of the pattern (§2.3), default 1/16 and 100 %.
   Include 1/32, 1/16T and 1/32T grids for fast breaks and triplets.
   `T` means triplet. Keep raw timing when the owner changes the grid.
5. Gate pads: the release writes `len`.
6. Erase: while erase is on and a pad is held, the hits of that pad that the playhead passes are
   removed. Keyboard: hold Backspace and the pad. The UI removes hits in the tick window between
   two frames; this is exact enough because the hits are removed, not played.
7. Not more than 4096 hits in a pattern: further hits are not recorded, the status says so.
8. The engine records the hit at the block start (error up to one block, §1). Write this into the
   spec. Change 101 removes the error if the owner hears it.

Engine: the `Recorded` queue; `lap` counting; `skip_lap` (§2.5); `CountIn`.

App: merge recorded presses and releases, add the undo step type and erase by the transport tick.
Add the new quantize grids to keyboard selection and controller controls, with visible rate feedback.

Keys: the new action `pattern.record`, keyboard `Ctrl+R` on every screen (reserved by 83a);
remove the old microphone moved-key notice when this binding ships. Microphone hold stays
on `Alt+R` after 83a. Check owner keymap overrides;
`Enter` on the Pattern screen comes in 98.

SMC-PAD: the context action `record.toggle` of RECORD tap becomes: the pattern recording when the
transport plays or the Pattern screen is open; the microphone recording otherwise. Update the
table in `SMC-PAD-INTEGRATION.md` §2.1 and its test. Erase from the controller comes with the
Pattern panel in 98.

Visual: the `COUNT n` badge with big digits in the strip during the count-in; the `● REC A1`
badge; notices: the hit limit (Warn), dropped recorded hits (Warn), undo of a pass (Done).

Tests:

1. Render: presses at off-grid frames while recording, 1/16 at 100 %: on the next lap the hits
   play on the grid ±1 frame; with Off they play where they were pressed.
2. Render: a hit 20 ms before a grid line: it sounds once in that lap (no double), and on the
   grid in the next lap.
3. Overdub keeps the old hits; undo removes only the last pass.
4. Erase removes only the held pad and only where the playhead passed.
5. Count-in: one bar of clicks, then recording starts at tick 0.
6. Parity: recording from keyboard pads and from SMC-PAD pads gives the same pattern.
7. Render and record 1/32, 1/16T and 1/32T at 172 BPM, including hits near the loop boundary.
   Check exact timing, no double hits and grid changes without loss of raw timing.

Render for the owner: `renders/97-pattern-record.wav` (raw take, then the quantized loop).

### Change 98 — `98-pattern-screen`

Goal: the Pattern screen (F5) with the step grid and the step mode (TR-REC).

Spec deltas: `patterns`; `screen-layout`; `screen-help`; `keyboard`; `command-panel` (Pattern
cells); `midi-learn` (PATTERN group).

Screen (register it in the screen registry of 86):

```
 A1  ▶ 2.3   120.0 BPM   ● REC   Q 1/16 100%   2 bars              bar 1/2
 ───────────────────────────────────────────────────────────────────────────
  1 kick    █ · · · █ · · · █ · · · █ · · ·
  2 snare   · · · · █ · · · · · · · █ · · ·
  3 hat     █ · █ · █ · █ · █ · █ · █ · █ █
  Q chop3   · · · · · · ▪ · · · · · · · · ·
            ▲
 ───────────────────────────────────────────────────────────────────────────
 [the pad grid as on F2; Ctrl+O hides it]
```

- Rows: the pads with hits in this pattern, plus the selected pad. The default view has
  16 steps of 1/16, one bar per page. `█` means a hit, with brightness for velocity.
  `▪` means an off-grid hit, `·` means empty, and the playhead column is lit.
- Add a 1/16 ↔ 1/32 view switch on keyboard and controller.
  Default proposal: 16 steps per page, with two pages per bar at 1/32.
  The alternative is 32 steps per bar page. Ask the owner before 98.
  Show the division and page span. Changing the view does not quantize stored hits.
- Step mode: the title shows `STEP` in the mark color, and the pad grid changes color.
  A pad hit toggles its step on the current page for the **selected** pad row.
  Adding a step plays the selected pad once. Removing it is silent.
  Keyboard Shift gives 127. An ordinary controller hit keeps its velocity.
- Add quick Accent and Ghost choices for the next inserted step from either input.
  Define the two velocities and control paths in the 98 design.
  Use right-half commands or controller actions, not commands on plain or Shift pad-block keys.
  Show the active choice. Shift on a pad still gives full hit velocity.
- Keys (right half only): `Space` play / stop. `Enter` record toggle. `J` / `L` step page.
  `I` / `K` selected row; `U` step mode on / off; `Y` next quantize; `Alt+Y` next strength
  (candidate: check clashes; `Ctrl+Y` is reserved for global redo by 83a);
  `Backspace` held + pad: erase; `Ctrl+Backspace`: clear the selected pad's hits (question);
  `Ctrl+D`: double the length (the hits are copied). Check every key against `config/keymap.toml`.
- F5 opens the screen (`screen.pattern`). F4 stays FX.

SMC-PAD:

- Panel Pattern cell opens the screen.
- Pattern context cells: c5 Step (latch), c6 Erase (latch: then a held pad erases), c7
  Quantize next, c8 Metronome, c9 Double length, c10 Clear pad (question), c11 Clear pattern
  (question), c12 Count-in on / off.
- LEFT / RIGHT change the step page. PLAY controls transport. RECORD controls recording.
  Knobs follow §2.3 of `SMC-PAD-INTEGRATION.md`: tempo, swing (later), quantize, strength, length.
- Selected row from the controller: hit the pad while step mode is off (it sounds and selects).

Visual: the `STEP` and `ERASE` badges, the step and erase pad glyphs, the grid tint
(`VISUAL.md` §2.2); hints for both inputs.

Tasks: add the view switch, division/page indicator and Accent/Ghost choices to step editing.
Keep the new paths in the reach table and the help for both inputs.

Tests: step toggles, the selected row and both page divisions through the parity harness.
Check both halves of a 1/32 bar, Accent/Ghost velocity, Shift velocity and ordinary MIDI velocity.
Render a pattern made only in step mode. The reach test includes the new actions.

Planned render: `renders/98-pattern-screen.wav`, with accents, ghosts and a 1/32 fill.

How to verify: make a one-bar beat through the keyboard, then through the controller.
Switch the view to 1/32. Add ghost snares and accents on both pages.
Listen to the render for the quiet hits and the fill.

### Change 99 — `99-pattern-slots`

Goal: 16 pattern slots, musical switch boundaries, a one-pass fill, copy and clear.

Spec deltas: future `patterns`, `projects`, `screen-layout`, `screen-help`, `keyboard`,
`midi-learn` and `command-panel`.

Behavior:

1. Slots `A1`–`A16` (the letter leaves room for more banks later). One plays at a time.
2. Slot pick: `P` on the Pattern screen or the panel cell → `Mode::SlotPick`; the pad grid shows
   the slots (empty, has hits, playing, queued); a pad hit picks the slot.
3. While playing, a picked slot waits for the selected boundary. The queued slot blinks.
   Keep loop-end and next-bar choices. Add boundaries of 4, 8 and 16 transport bars.
   Default proposal: 8 bars. The setting uses the transport bar grid, not the pattern length.
4. A fill slot plays once, then returns to the slot that played before it.
   Define the return phase, cancellation and another queued selection in the 99 design.
5. Copy: pick the source, panel Copy, pick the target. Ask if the target has hits.
6. Clear a slot: ask first.

Tasks: add boundary selection and fill selection to keyboard and controller paths.
Save the boundary setting and fill state needed for project restart under the 99 design.
The engine switches on the exact boundary frame.

Visual: `SLOT`, `NEXT A2` and `FILL` badges, slot pad glyphs and the queued-slot blink.
Show the selected boundary. Follow `VISUAL.md` for both inputs.

Tests: render switches at 4, 8 and 16 bars, plus loop end and next bar.
Check no lost or doubled hit at each seam. Test selection before, on and after a boundary.
Check fill play once and return, cancellation and another queued selection.
Keep the queue-cancel test and the session round trip of 16 slots.

Planned render: `renders/99-pattern-slots.wav`, a main slot, one fill and the return.

How to verify: select an 8-bar boundary and queue another slot from each input.
Trigger a fill. Listen for its single pass and the return to the previous slot.

### Change 100 — `100-note-repeat`

Goal: note repeat (roll) on the transport grid.

Behavior: while repeat is on and the owner holds a pad, the engine repeats it on the transport grid.
Rates include 1/8, 1/16, 1/32, 1/16T and 1/32T, with a rate knob.
The default velocity stays at the first hit's velocity.
An optional velocity ramp gets louder or quieter over a set length.
Define ramp length, endpoints and restart rules in the 100 design.
While recording, the repeated hits and their velocities become pattern hits.
Keyboard: hold `N` on the Pattern screen.
SMC-PAD: use a Pattern panel cell or a hold binding chosen in the design.
Do not use the device's Note Repeat button. Its clock is not the ptsr clock.

Spec deltas: future `patterns`, `pad-playback`, `screen-help`, `midi-learn` and `command-panel`.

Tasks: add the rates and velocity ramp to the engine and both control paths.
The engine already tracks held pads through press and release commands.
Keep callback work bounded and free of allocation.

Visual: `REPEAT 1/16`, the rate in the value pop-up and the ramp direction/state.

Tests: render timing within one frame at all rates, including triplets at 172 BPM.
Check rising, falling and fixed velocity, recording, release and a repeat-rate change.
Benchmark dense rolls against the callback budget gate.

Planned render: `renders/100-note-repeat.wav`, fixed, rising and falling rolls.

How to verify: hold a roll from the keyboard, then the controller.
Select each new rate and both ramp directions. Record a roll and play it again.
Listen for even triplets and the intended change of velocity.

### Change 101 — `101-input-timestamps` (only if needed)

Goal: remove the error of up to one block for live hits (§1). Do it only when the owner hears
the error with quantize Off.

Design: the audio thread writes `(frames_at_block_start, Instant)` at each callback (a seqlock
or two atomics with a generation). The input threads compute the frame of a hit from `Instant`.
`PadPress` gets `at: Option<u64>`. The engine plays a stamped hit with a fixed delay of one
block (constant latency instead of a variable one) and records its exact tick. Tests with a
synthetic clock.

## 4. Additions

`CROSS-CUTTING.md` §4 adds tasks to 95–98 (tap tempo, metronome level, project switch, length
change, closed passes, undo of step edits, rows of other banks) and the change `100a-pad-mute`.
They are part of these changes.

## 5. Later, not planned here

Groove templates, pattern variations, pattern chains and recording takes remain later work.
They reuse the raw hit model of 96. Each needs its own OpenSpec change before code.
They are not part of 95–101.

Change 104 covers parameter locks, including pitch, start point, filter and sends.
See [SAMPLE-WORKFLOWS.md](SAMPLE-WORKFLOWS.md). Step velocity belongs to 98.
These are not unassigned later work.

Swing (in `compile`), per-step probability, micro shift, reverse and ratchets remain outside this track.
The multi-pattern launcher, song mode, MIDI clock with the SP-404 and other time signatures also remain later work.
