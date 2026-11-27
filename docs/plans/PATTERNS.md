# Plan: transport, patterns, recording and the sequencer (changes 95–101)

Read `README.md` first. Key choices follow `KEYMAP-CLEANUP.md` (83a). This track needs
the input work (84–90), the SMC profile and panel
(91–92), the reach checks (93) and comfort (93a, including redo). Pad lights (94) are optional.
The owner deferred separate sound-quality work, including callback investigation. It no longer precedes patterns by default.
Existing real-time rules, tests and benchmark limits still apply. Each proposal must resolve its remaining input prerequisites.
Each change states its keyboard keys and its SMC-PAD path. The reach test of 93 covers them.
The design sketches below are proposals, not existing Rust interfaces or saved file formats.

The additions below support fast breaks, fills and detailed step control inside 95–100.
[SAMPLE-WORKFLOWS.md](SAMPLE-WORKFLOWS.md) continues with sample tempo and sound work in 102–108.

## 1. Where we start

- Transport 95 is implemented and archived. It provides an audio-frame clock at 960 ticks per quarter note, 4/4 and 40–300 BPM.
  Space toggles the clock on Pads, FX and Settings. Chop and Files keep their previews.
  Format 4 saves tempo and metronome preference. It never restores running state or position.
  Start 96 from this implementation and the [main transport specification](../../openspec/specs/transport/spec.md).
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

The implemented transport advances fractional ticks for each processed audio frame. Empty blocks do not advance it.
For 96, use that clock to schedule events within each block. Do not add a second clock.
The scheduling sketch uses `offset = round((t - tick_at_block_start) * frames_per_tick)` for integer output-frame offsets.
A tempo change acts at the next block start. The tick position does not jump. 4/4 only, 40–300 BPM.
Define block-edge rounding and loop behavior in the 96 design before code. Fractional source starts are not a prerequisite.

### 2.3 A pattern stores ticks and settings for the engine

App side (`crates/app/src/pattern.rs`), saved in the project:

```rust
pub struct PatternData {
    pub bars: u8,              // 1..=64
    pub quantize: Grid,        // Off, 1/4, 1/8, 1/16, 1/32, 1/64, 1/8T, 1/16T, 1/32T
    pub strength: u8,          // 0..=100, steps of 10
    pub hits: Vec<Hit>,        // saved ticks, with up to 4096 rows
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

`compile` leaves out saved hits at or beyond the current pattern length. Those hits stay
saved and return when the length grows. It plays each visible hit at its saved tick. New
recordings apply quantize and strength when they capture a hit. The pass saves that tick.
Changing the grid or strength affects later hits only. The raw played time is not saved
after the pass. Quantize Off keeps the accepted tick. If capture moves a visible hit to
the loop end, that hit wraps to the start. Same-pad hits on one tick play once by the
specified collision order. A later swing feature must state whether it changes saved
ticks or acts at playback.

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

Transport 95 publishes one coherent snapshot with a revision and project epoch. The UI never advances its own clock.
Keep that contract in 96. Do not replace it with unrelated atomic reads that can mix projects or clock states.
Recording and lap fields remain future work for 97. Define their coherent publication before code.

### 2.7 What a pad hit means in the new modes

Add to `Mode` (change 85 made it): `Step` (a pad hit toggles a step, routed to the UI, no fast
sound), `Erase` (a held pad erases its hits under the playhead, no sound), `SlotPick` (a pad hit
picks a pattern slot, no sound). `Record` is not a mode: pads sound and the engine records them.

## 3. The changes

### Deferred — `94a-callback-budget`

Goal, when resumed: investigate the earlier callback peaks above 50 % and slow-core CPU spikes.
The owner deferred this work until a separate request. It is not a prerequisite for transport or pattern implementation.
Historical measurements remain unresolved, not accepted as fixed.
Existing checks still apply to each pattern change. This deferral does not permit a failed-check commit.

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

Owner check, when resumed: review the repeated measurements and the unresolved limits.
If a sound path changes, render it and give the owner a listening comparison.

### Change 95 — `95-transport` (implemented and archived)

The [archived proposal](../../openspec/changes/archive/2026-11-08-95-transport/proposal.md) and main specifications define the implemented behavior.
They replace the imported 95 sketch, not the requirements of later changes.

- Audio-frame clock: 4/4, 960 ticks per quarter note, 40–300 BPM in 0.1 BPM steps, default 120 BPM.
- Starting from stopped resets the clock. Stop resets the clock. Repeated explicit play/stop commands are idempotent.
  One-shot tails continue. Tempo changes preserve position.
- A dedicated metronome uses the existing click sample without taking a pad voice.
  Accents are 1.0 on beat 1 and 0.6 on other beats. Onset accuracy is within one output frame.
- Space uses `transport.toggle` on Pads, FX and Settings. Chop and Files keep their preview actions.
  The legacy `transport.play_stop` still means Chop preview. `metronome.click` remains the test-click action without a new default key.
- Ctrl+Up/Down changes tempo by 1 BPM. Ctrl+Shift+Up/Down changes it by 0.1 BPM. Alt+M toggles the metronome.
- Existing MIDI learn supplies GLOBAL buttons and an absolute tempo knob through the current takeover rules.
  No SMC-PAD hold or factory role is implemented by 95. Those controls still need the measured profile and hold support.
- Format 4 saves `bpm` and `metronome`, not running state or position.
  Formats 1–3 remain readable. Older binaries refuse format 4. Keep backups before saving.
- The title shows accepted clock state. Tempo changes show the BPM value and range.
  Published snapshots retain project epochs and coherent fields.

Pattern scheduling is not implemented. Count-in belongs to 97. Half-time remains `1.x` under V1.
Tap tempo and metronome level remain planned additions, not implemented 95 controls.
Keep their scope and input decisions in a later approved proposal.
Separate metronome traces on the waveform and diagnosis of the reported Chop-start click remain deferred.

Offline transport, input, migration, title and no-allocation tests pass. The p99 benchmark exit checks pass.
Recorded maximum-budget failures remain unresolved. General acceptance does not establish individual terminal, controller or listening results.

Local render: `renders/95-transport.wav`.
See the [separate owner procedure](../LIVE-CHECKS.md#95-transport-and-metronome) for keyboard, controller and listening checks.

### Change 96 — `96-pattern-play`

Goal: one pattern plays in a loop with the transport. No recording and no screen yet; the owner
writes a pattern into `project.toml` by hand to check it.

Spec deltas:

- New capability `patterns`.
- `projects`: saved patterns and migration from implemented format 4 and supported earlier formats.
- `pad-assignment`: swap moves the hits.
- `pad-playback`: the slice pitch gate.

The per-pad slice pitch gate belongs here because it needs the pattern clock.
It defines planned playback behavior, not a general sound-quality correction.

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
6. Slice pitch gate: a per-pad setting, on by default for slices, cuts a pitched-up slice at the next pattern step.
   Never extend the slice beyond its source end. Use the engine's existing cut path.
   Define which step grid supplies the boundary before code, including empty steps, triplets, swing and loop end.
   Also define stop, pattern replacement, tempo changes and interaction with manual hits.

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

2. Migrate from format 4 and supported earlier formats without patterns. Preserve saved tempo and metronome preferences.
   Use 120 BPM only when the older format has no tempo. Choose the new format version in the 96 proposal.
   Preserve existing mute-group numbers and memberships, manual assignments and ungrouped overlap.
   Add keyboard and controller paths for the per-pad pitch gate, with saved state and undo.
3. `Ctrl+X` swap: hits of the two pads trade pads in every pattern (if the owner confirmed).
4. The title line shows the slot, for example `A1 ▶ 1.3`.

Visual: the slot in the `▶` badge and the transport segment (`▶ 120.0 · A1 2.3`).

Tests:

1. Render: a 2-bar pattern at 120 BPM: each hit at its frame ±1, over 4 loops.
2. Render: a gate pad hit with `len`: the release at its frame.
3. Render: pattern replaced while playing (one hit added ahead of the playhead): no hit missed
   or doubled around the swap.
4. Render: mute group pads in a pattern cut each other.
5. Session round trip and migration fixtures for every supported earlier format, preserving transport preferences and existing mute groups.
6. Pattern loops and gate releases keep the transport clock and metronome timing unchanged.
   Half-time is outside the 1.0 scope. Do not add it as a prerequisite for 96.
7. Benchmark dense patterns at 172 BPM against the existing callback limits.
8. Render pitched-up slices with the pitch gate on and off. Check the next-step cutoff and its fade.
   Cover empty steps, loop end, stop, tempo changes and pattern replacement under the approved boundary rule.
9. Check pitch-gate controls, undo and old-session defaults through keyboard and raw MIDI production routes.

Render for the owner: `renders/96-pattern-play.wav`, including separate pitch-gate comparisons.

How to verify (owner): paste the TOML above into the project, load the kick on pad 1 (bank 1,
`1`) and the snare on pad 2 (`2`), start with Space.

### Change 97 — `97-pattern-record`

Goal: live recording into the current pattern with overdub, quantize, erase, undo of a pass,
and a count-in.

Spec deltas: `patterns` (record, overdub, quantize, erase, undo, count-in, limits); `keyboard`;
`midi-learn`; `command-panel` (Pattern cells start in 98; here RECORD is enough).

Behavior:

1. Record toggle while stopped: one bar of count-in with the click, then play and record from
   tick 0. While playing: record from now. On A1, Esc or command P1 cancels count-in and keeps the overview open.
2. Overdub always: new hits add to the old.
3. A pass is from record on to record off; one pass is one undo step (`Ctrl+Z`, panel Undo).
4. The quantize and strength are settings of the pattern (§2.3), default 1/16 and 100 %.
   Include 1/32, 1/16T and 1/32T grids for fast breaks and triplets.
   `T` means triplet. A grid change leaves saved hits at their captured ticks.
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
   Check exact timing, no double hits and saved ticks after grid changes.

Render for the owner: `renders/97-pattern-record.wav` (raw take, then the quantized loop).

### A1 step timeline (172)

Show the selected bar and global step with their totals in the overview and Step Edit.
Use a second line for the view grid and the quantize of the next live recording.
Show the current playback position separately from the selected column.
Keep the selected time when the owner enters Step Edit or returns to the overview.
See [the keyboard and SMC-PAD routes](../CONTROLLER-PATHS.md#select-and-edit-an-a1-step).

### A1 transport feedback (164)

Show First pass or Overdub from the hits present at the start of the pass.
Show the accepted pause position and name the control that resumes it.
Show Stopped when the transport returns to the start.
Put the first keyboard tap and the new BPM in the notice row.

### Narrow A1 panel (163)

Use the A1 grid area for the command panel at 80x26. Keep the title and status.
Show full Help and the project picker above that panel.
Give the controller status one row and short help two rows.
Restore the grid when the panel closes.

### A1 Setup help (165)

Show the Setup route before A1 exists. Show the current row action and the close control.
Show the Setup keys first in F1. Give the clear question its own controls.

### A1 overview controls (154)

In the A1 overview, press J/L or Left/Right to move one step. Press I/K or
Up/Down to move one bar. On SMC-PAD, turn E1 for a step. Close the command
panel to use LEFT and RIGHT for bars. Press F2 to edit a pad.

### Select one hit in Step Edit (156)

If a step contains more than one hit on the selected pad, press I/K to
select one. On the SMC-PAD panel, use P14 for the previous hit and P15 for
the next hit. The hit number and velocity are visible with the panel open.

### Change 98 — `98-pattern-screen`

The [pattern track](#pattern-track-owner-review-2026-11-27) replaces the screen sketch and the keys of this section.
Keep this section as history of the 98 decisions.

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

### Change 98c — `98c-quantize-on-input` (owner decision 2026-11-12, after 98b)

Owner problem: a quantize change moves every saved hit, also the hits that were already in
the correct place. Owner decision: quantize as on an MPC or SP-404. This replaces the
non-destructive quantize of section 2.3 for recorded hits.

- A hit gets the quantize and strength of the moment of its capture. The pass stores the
  quantized tick. The saved format stays the same.
- A change of Quantize or Strength applies only to the next hits. Saved hits stay in place.
- The raw played time is not kept after the pass. Pass Undo still restores the earlier rows.
- One pad in one quantized cell keeps one hit (owner decision 2026-11-12). The last
  captured hit replaces the earlier one, in one pass and across passes. The `2-9` cells then stay
  only for quantize Off, strength below 100 % and old projects. A flam of one pad in one
  cell needs a finer grid or quantize Off.
- A later explicit command can quantize all of A1 with a question. It is not part of 98c.
- Show the quantize of the next hits in the strip header: `view 1/16 · quantize 1/4 100 %`.
  Proposed, not yet accepted by the owner.

Spec deltas: `patterns` (96 compile, 97 capture and settings), `screen-layout` (header),
`screen-help`. Open questions for the design:
- old projects with raw ticks (default: load them as they are, no migration).
- Gate lengths.
- the controls text for Quantize and Strength.

### Pattern track (owner review 2026-11-27)

The owner used the pattern work on `master` at `97cdda8`. The result is not clear and not
fast on the keyboard or on the SMC-PAD. The screen looks poor, and Help does not help.
The owner approved an interactive mockup of the new screen. This track replaces the
`pattern-screen` item of [UI-UX.md](UI-UX.md#31-pattern-track).

Checked problems at `97cdda8`:

- The pattern is in five places: the mini lane on Pads, the F5 overview, step edit on F7,
  the setup panel on `Alt+N` and global chords (`config/keymap.toml`: `Alt+E`, `Alt+N`,
  `Alt+Z`, `Alt+B`, `Alt+Up`, `Alt+Down`, `Alt+H`).
- F5 is a mode of Pads. F7 works only from the overview.
- `Ctrl+Z` and `Alt+Z` are two undo keys with different histories.
- The screen says `A1` on each label, but only one pattern exists.
- To write four kicks, the owner moves the cursor four times. No mode makes the 16 pads
  the 16 steps of one sound.
- The header uses lines such as `BANKS · pads with A1 hits` (`crates/tui/src/pattern.rs`).
  The `~` glyph (`crates/tui/src/pattern.rs`) has no explanation on the screen.
- F1 starts with the Pitch Gate row (`crates/app/src/help.rs`).

- The catalogue entry `pads-pattern-overview-straight-120x40` draws a frame for each bank.
  One pad row uses three lines. The grid starts near column 46.
- The catalogue entry `pads-help-120x40` has 78 rows. Undo and redo are in the CHOP group.
  Tap tempo is in the SOUND group.

Goals:

- One place for the pattern: the F5 screen, with plain keys of the right half.
  Common work needs no Alt chord.
- Two clear ways to write: play it (record) and step it (pads as steps).
- The grid is the screen: large cells, visible velocity, one header line.
- The same model on the SMC-PAD.
- Help answers tasks first.

#### The pattern screen

```
 F5 PATTERN  ▶ 1.3  2 bars  120 BPM  bar 1/2            [STEP · pads = steps of kick]
            1  ·  ·  ·  2  ·  ·  ·  3  ·  ·  ·  4  ·  ·  ·
 1 kick     █  ·  ·  ·  ·  ·  █  ·  █  ·  ·  ·  ·  ·  ·  ·
 2 snare    ·  ·  ·  ·  █  ·  ·  ·  ·  ·  ·  ·  █  ·  ·  ▪
 3 hat      ▓  ·  ▪  ·  ▓  ·  ▪  ·  ▓  ·  ▪  ·  ▓  ·  ▪  ·
 Q chop 3   ·  ·  ·  ·  ·  ·  ▓  ·  ·  ·  ·  ·  ·  ·  ·  ·
 velocity   ▆  ▁  ▁  ▁  ▁  ▁  ▆  ▁  ▆  ▁  ▁  ▁  ▁  ▁  ▁  ▁
 step 1.1 · kick · loud (vel 127)
 setup  bars 2 · tempo 120 · grid 1/16 · record quantize 1/16 100 % · metronome on · count-in 1 bar
```

- **Header:** one line. It shows the screen badge, the transport state and position
  (bar.beat), the length, the tempo, the bar page and the mode badge.
- **Rows:** the pads with hits and the selected pad, in bank order. No bank frames.
  The pad label uses its bank tint. A thin rule separates banks only when two banks show.
  `Alt+H` shows all rows.
- **Columns:** the grid uses the full width. A step cell is at least 3 columns wide and
  grows with the terminal. The header numbers the beats (`1 · · · 2 · · ·`).
  Keep the beat shading. Show one bar per page at 1/16. The design selects half a bar per
  page or narrower cells for finer grids. All cells on one page have the same width.
- **Velocity:** three different levels (fill or brightness). The velocity lane under the
  grid shows the selected row. Later step values (locks, ratchets, probability in 0.4b)
  use this lane.
- **Detail line:** the selected step, the pad and its content: `no hit`, the velocity,
  `off grid +12 ticks` or `2 hits`. The glyphs can stay. The detail line and Help explain them.
- **Setup line:** bars, tempo, view grid, record quantize and strength, metronome and
  count-in. It is always visible. Count-in stays one bar. The line shows it and does not change it
  (owner decision, 2026-11-27). `N` moves the focus to it: `I` / `K` select, `J` / `L`
  change, Esc goes back. It replaces the `Alt+N` panel.
- **Playhead:** a lit column during play. The edit cursor has its own mark.
- **Empty pattern:** two lines, `Ctrl+R records what you play` and `Enter writes steps`.
- **Words:** `pattern` on all labels. A slot name shows only after slots exist (99).

#### Two ways to write

1. **Play it.** `Ctrl+R` (RECORD on the SMC-PAD) starts a pass with the count-in.
   The pads play and the pass records them. `Ctrl+R` ends the pass.
   The header shows `● REC 1.2`. The result notice gives the hit count and the undo key.
2. **Step it.** `Enter` starts and ends step edit. F7 stays as a second key for one
   release with a moved-key notice. Then F7 is free.
   - **Pads = steps** (default in step edit): each of the 16 pad keys is one step of the visible bar.
     The 16 SMC-PAD pads act the same. The steps belong to the selected row. A hit toggles the step.
     Keyboard Shift gives full velocity. The SMC-PAD gives the hit velocity.
     The header shows the key of each step above its column. `I` / `K` select the row.
     To add a row for a new pad, select that pad in pads = sounds.
   - **Pads = sounds:** the current behavior. `J` / `L` move the time column.
     A pad toggles its own hit there.
   - `Tab` switches the two. The mode badge always names the active one:
     `STEP · pads = steps of kick` or `STEP · pads = sounds`.

The pattern screen keys and their conflicts are in
[KEYMAP-CLEANUP.md](KEYMAP-CLEANUP.md#8-pattern-screen-keys-pattern-track).

#### One undo

`Ctrl+Z` undoes the last action of the owner, also a pattern pass or a step.
If the program keeps two histories inside, it orders them by time for the owner.
Keep the existing safety rule: a pattern undo does not cross a newer structural edit.
In that case, the notice tells the reason. `Alt+Z` stays for one release with a moved-key notice.

#### Pads screen

The Pads strip keeps a small pattern lane: the used rows that fit, with the playhead.
It shows one hint, `F5 pattern`. Pads has no pattern settings and no pattern edits.

#### The five changes

The owner split the track into five changes on 2026-11-27. Each change keeps every current
keyboard and SMC-PAD path until a later change replaces it.

Do them in this order. Each change gets the next free OpenSpec number, its own branch and its own PR.
Write the specifications first. Add catalogue fixtures at 120×40 and 80×26, and keyboard and raw MIDI tests.
Start the next change only after owner acceptance.

1. `pattern-screen-layout` (174): F5 as its own screen with the layout above. The setup line
   only shows values. The old overview goes. The `Alt+N` panel, the SMC-PAD setup cell, F7 step edit,
   live record, erase and tap tempo keep their current keys on the new screen.
   The empty screen names F7 for steps. Pads keeps the mini lane with the hint `F5 pattern`.
2. `pattern-setup-line` (175): `N` moves the focus to the setup line. It replaces the `Alt+N` panel.
   SMC-PAD knobs on F5 change bars, tempo, view grid, record quantize and strength directly.
   Clear moves to the pattern screen.
3. `pattern-step-input` (176): step edit with pads = steps and pads = sounds,
   the pattern screen keys and one undo.
4. `pattern-smc` (177): the rest of the SMC-PAD model of
   [SMC-PAD-INTEGRATION.md](SMC-PAD-INTEGRATION.md#25-the-pattern-screen-pattern-smc),
   with routes in `docs/CONTROLLER-PATHS.md` and device checks.
5. `help-tasks` (178): Help by tasks, see [UI-UX.md](UI-UX.md#31-pattern-track).

Renders: change 3 renders a beat written only in pads = steps, with soft, medium and loud hits.
Changes 1, 2, 4 and 5 do not change sound. Each proposal states this in one sentence.

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

`CROSS-CUTTING.md` §4 proposes tap tempo, metronome level, project switch, length changes,
closed passes, undo of step edits, rows of other banks and `100a-pad-mute`.
Transport 95 did not implement tap tempo or metronome level. Assign those additions in a later approved proposal.
V1 selects the 1.0 scope. The active change must state its exact tasks before code.

## 5. Later, not planned here

Groove templates, pattern variations, pattern chains and recording takes remain later work.
They reuse the raw hit model of 96. Each needs its own OpenSpec change before code.
They are not part of 95–101.

Change 104 covers parameter locks, including pitch, start point, filter and sends.
See [SAMPLE-WORKFLOWS.md](SAMPLE-WORKFLOWS.md). Step velocity belongs to 98.
These are not unassigned later work.

Swing (in `compile`), per-step probability, micro shift, reverse and ratchets remain outside this track.
The multi-pattern launcher, song mode, MIDI clock with the SP-404 and other time signatures also remain later work.
