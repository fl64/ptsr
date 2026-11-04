# Plan: sample tempo, patterns and sound (changes 102–108)

Read [README.md](README.md) first. [VISION.md §16b](../VISION.md#16b-sample-workflows-tempo-breaks-bass-and-mix)
gives the musical goal. This plan is a guide, not requirements or shipped behavior.
Create an OpenSpec proposal, design, spec deltas and tasks before each change.
Activate one change at a time. Check `AGENTS.md` before implementation.

## 1. Goal and order

Make breaks and bass at 170–180 BPM without a mouse. Keep the keyboard controls and
provide a controller path for each action that does not need text.

The base is transport, patterns and pad mute through 100a. Change 101 is optional.
The callback budget gate before 95 also applies to this track.
The saving plans 86b–86d protect projects before this track adds more saved data.

- 102 needs transport 95 and comes after the pattern base in this roadmap.
- 103 needs sample tempo 102, pattern playback 96 and the step screen 98.
- 104 needs the hit model of 96 and step control from 98.
- 105 needs transport and patterns. It builds on the existing `record.output` path.
- 106–108 can change order between them after the pattern base.
  Change 108 needs the transport clock, but not sample tempo 102.

Numbers identify changes, not permission to implement them together.
Later capabilities do not block an earlier independent change.

## 2. Rules for each change

- State the keyboard path and the controller button, panel cell or knob path in the spec.
  Update the reach checks of 93. New keys stay outside the pad block.
- Use `VISUAL.md` for badges, pad states, notices, value changes and hints for both inputs.
  Color alone does not identify a state.
- Test shared actions with keyboard events and raw MIDI through the production profile and dispatch.
  Preserve controller velocity unless an explicit action changes it.
- Keep allocation, locks, I/O, logs and panics outside the audio callback.
  Benchmark each new sound path with 32 voices at 128 frames / 48 kHz.
- Give every sound change a WAV with automatic checks for finite values, clipping,
  expected silence and hit timing within one sample of the reference schedule.
- Add a migration before a saved format changes. Old projects retain their current sound
  when new settings are absent or off.
- Add separate keyboard and controller owner checks to `docs/LIVE-CHECKS.md`.
  A render test does not prove a hardware or listening pass.

The control paths below describe planned operations, not existing keys or panel cells.
Each change's design selects exact controls before code.

## 3. The changes

### Change 102 — `102-sample-tempo`

Goal: fit a break recorded at 130–160 BPM to a project at 170–180 BPM.
Start with repitch, where speed and pitch change together.

Behavior:

- A sample has `bpm` and `beats`, its length in beats. Slices inherit their sample's tempo.
- The owner can set the length to N bars. In 4/4, `beats = 4 × bars` and
  `bpm = beats × 60 / duration_seconds`. This is the preferred method for known loops.
- Automatic detection proposes a tempo. The owner can accept it or correct it with ×2 or ÷2.
  Detectors can confuse 87 and 174 BPM. A knob also sets the value directly.
- Chop offers the tempo grid, including 1/8 and 1/16, next to chop by hits.
- A pad fit setting is `off` or `repitch`. The default for a new slice is off.
  In repitch, `speed = project_bpm / sample_bpm`. Pitch rises with speed.
- A project tempo change updates pads with repitch on. Fit off keeps the existing playback.
  Missing or invalid tempo does not cause division by zero or an unannounced speed change.
- Time stretch without a pitch change is later work, not part of 102.

Spec deltas: `sample-loading`, `chopping`, `pad-playback`, `projects`, `editing`,
`screen-help` and the future `transport` capability.

Tasks:

1. Define metadata ownership, the saved representation and the migration in the design.
   Keep the source audio file unchanged. Define how fit combines with the existing pad pitch.
2. Add length-in-bars entry, a tempo knob and the detection proposal with ×2 and ÷2 corrections.
   Default detector candidate: aubio. Check its version, wrapper and license before code.
3. Add tempo-grid chopping and tempo inheritance for slices, including older chop variants.
4. Add fit off / repitch and smooth tempo changes. Show sample BPM, project BPM and fit state.
5. Add keyboard value controls and learned controller controls. Update help and reach checks.

Tests: known loop lengths, the 87/174 correction, silent input and invalid metadata.
Check old projects, all saved chop variants and project moves.
Render breaks at several source tempos into a 172 BPM project.
Check duration and the expected pitch ratio. Check a project tempo change without a click.

Planned render: `renders/102-sample-tempo.wav`, with fit off, repitch and a tempo change.

How to verify:

1. Keyboard: load a known four-bar break. Set its length to four bars. Set the project to 172 BPM.
2. Turn repitch on. Play the break with the metronome. Change the project tempo.
3. Controller: repeat these operations through the tempo controls, fit control and Chop grid control.
4. Listen to the planned render. Check the beat fit and the expected pitch rise.

### Change 103 — `103-break-to-pattern`

Goal: turn a chopped break into a pattern with its original groove, then rearrange it.

Behavior: each slice becomes a hit at its position on the sample's tempo timeline.
Use sample BPM to convert positions into pattern ticks.
Keep off-grid groove within the pattern resolution. Do not force every hit to an equal-parts grid.
The owner can then change the slice order in step mode.

Spec deltas: `chopping`, future `patterns`, `projects`, `screen-help` and `command-panel`.

Tasks:

1. Define the sample range, pattern length and rounding rules from sample positions to ticks.
   Define slice-to-pad assignment without an unannounced replacement of occupied pads.
2. Add the conversion action with a keyboard path and a controller panel path.
   Ask before replacement of a pattern that contains hits. Make conversion one undo step.
3. Show the target slot and the result. Preserve slice links and the chosen chop variant.

Tests: uneven slice positions, a selected range and two chops of one sample.
Check conversion, rearrangement, undo and project restart through both inputs.
Compare playback with the reference tick map, within one sample of that map.
Do not claim that tick rounding preserves every original audio frame.

Planned render: `renders/103-break-to-pattern.wav`, original groove followed by a rearrangement.

How to verify:

1. Keyboard: chop a break by hits. Convert it to a pattern. Play it at the project tempo.
2. Rearrange two slices in step mode. Undo the change.
3. Controller: repeat conversion and rearrangement through the panel and step controls.
4. Listen for the original groove, then the changed slice order in the planned render.

### Change 104 — `104-parameter-locks`

Goal: give each step its own sound settings, as in VISION §12.

Behavior: a hit can hold locks for filter cutoff, resonance, pitch, start point, decay and sends.
In step mode, hold a step and turn a knob.
Locks affect that hit, not the stored pad values. The next unlocked hit uses the pad values.
The proposed first set is cutoff, pitch, start, decay and sends. Resonance remains in this change's plan.

Spec deltas: future `patterns`, `editing`, `pad-playback`, `projects`, `screen-layout`,
`screen-help`, `midi-learn` and `command-panel`.

Tasks:

1. Extend the hit model of 96 with an optional, bounded list of locks.
   Define saved defaults and migration. An absent list keeps the existing hit behavior.
2. Add step hold plus knob editing for the keyboard and controller.
   Define keyboard value keys and a clear-lock action. Keep Shift on pads as full hit velocity.
3. Show locked steps and the parameter value. Add undo and restart support.
4. Apply locks without changing other hits or allocating in the callback.
   Define how overlapping voices use different locks.

Tests: each parameter, an unlocked following hit, overlapping hits, lock removal, undo and restart.
Run the step-hold scenario through keyboard input and raw MIDI. Benchmark dense locked patterns.

Planned render: `renders/104-parameter-locks.wav`, a bass phrase with changing sound on each step.

How to verify:

1. Keyboard: select a bass step. Hold it. Change cutoff, pitch and start with the value controls.
2. Play the pattern. Remove one lock. Undo its removal.
3. Controller: repeat with a held step pad and the corresponding knobs.
4. Listen for changes on the selected hits only. Check the unlocked hits against the pad sound.

### Change 105 — `105-resample-in-tempo`

Goal: record, process, record again and chop, without a separate recorder.

Behavior: record a pad, the current pattern or the ptsr master for N bars onto a new pad.
Start and stop at bar lines. Print the effects into the new sample.
Keep the source pads, pattern and files unchanged.

The current `record.output` records the sound device's monitor, including other programs.
Reuse its controls and recording infrastructure, but define isolated internal sources for this change.
Do not label a device-monitor recording as an isolated ptsr master.

Spec deltas: `recording`, `pad-assignment`, future `transport` and `patterns`, `projects`,
`screen-layout`, `screen-help` and `command-panel`.

Tasks:

1. Define source selection, N-bar limits and the relation to the existing 30-second recording cap.
   Define tempo changes, half-time, stop, panic, cancellation and write failure during a take.
2. Add bar-line capture of each source with effects. Keep file writing outside the callback.
   Prevent the recorded destination from feeding its own capture.
3. Select a new pad from keyboard or controller. Ask before replacement of an occupied pad.
   Show the source, waiting state, bar count, completion or failure.
4. Save the new sample in the project's samples folder and retain relative paths.
   Check project deletion and disk failure without damage to the source or last saved project.

Tests: N-bar length and both boundaries within one sample, each source and printed effects.
Check isolated capture while another program's audio exists on the device monitor.
Check cancellation, limits and write failure. Render two resample passes followed by chopping.

Planned render: `renders/105-resample-in-tempo.wav`, source, first pass and second pass.

How to verify:

1. Keyboard: record two bars of a pattern to a new pad. Process that pad. Record it again.
2. Chop the second recording. Compare it with the unchanged source.
3. Controller: repeat source selection, bar count, destination selection and capture without text entry.
4. Listen for printed effects and exact bar length, without extra monitor audio or a feedback loop.

### Change 106 — `106-bass-play`

Goal: play bass notes from one sample, with mono play and glide.

Behavior: chromatic mode maps the 16 pads to notes of one sample in a selected scale.
Mono play makes a new note stop the previous note without a click.
Glide moves between note pitches over a set portamento time.
These are a small bass instrument, not the full keygroup and layer model of the vision.

Spec deltas: `pad-playback`, `input-devices`, `editing`, future `patterns`, `projects`,
`screen-layout`, `screen-help`, `midi-learn` and `command-panel`.

Tasks:

1. Define root note, scale, octave range, note recording and their saved representation.
   Do not assume that the pad-only hit model of 96 already stores notes.
2. Add chromatic entry and exit, mono selection and the glide-time control for both inputs.
   Show the mode, note names and mono/glide state on the grid and in help.
3. Define overlapping holds, releases, legato and project-switch behavior before engine code.
   Keep ordinary pad play unchanged outside chromatic mode.

Tests: note pitches, scale mapping, velocity, mono voice replacement and held-note release order.
Test glide duration, zero glide, restart and pattern note playback. Benchmark the new voice path.

Planned render: `renders/106-bass-play.wav`, separate notes, mono phrase and glide phrase.

How to verify:

1. Keyboard: choose one bass sample. Enter chromatic mode. Play a scale across the pads.
2. Enable mono play. Set glide time. Play overlapping notes. Leave the mode.
3. Controller: repeat with the note pads and the mono/glide controls.
4. Listen for the intended notes and smooth glide, without a stuck note or a click.

### Change 107 — `107-mix-base`

Goal: keep a dense mix clear and loud, with a clean mono sub.

Behavior: a pad or group can duck other pads or a bus through sidechain.
Kick and snare can control the ducking. Each pad has a high-pass filter.
The master has a compressor and limiter. The status line shows short-term LUFS loudness.

Pad high-pass filtering already exists. Preserve it and check its control paths instead of adding it again.
Define the clean mono-sub path and the minimum bus routing in this change's design.
A full track mixer is outside this change.

Spec deltas: `pad-playback`, `audio-engine`, `editing`, `projects`, `screen-layout`,
`screen-help`, `midi-learn` and `command-panel`.

Tasks:

1. Define sidechain sources, targets, amount, attack and release, with keyboard and controller paths.
   Keep sidechain off for old projects. Define the minimum routing before code.
2. Retain pad high-pass controls. Define sub cleanup and mono output without an unannounced change to existing stereo pads.
3. Add master compression and limiting with safe defaults, smooth changes and visible bypass or active state.
4. Add the short-term LUFS meter. Define its measurement reference, tolerance and silence indication.
   Keep meter work within the callback budget. Add state and value notices under `VISUAL.md`.

Tests: sidechain timing and recovery, independent targets, high-pass regression and mono sub output.
Check compressor response, limiter peaks and LUFS against reference signals with a declared tolerance.
Render loud overlapping drums and bass. Benchmark with the full active mix path and 32 voices.

Planned render: `renders/107-mix-base.wav`, dry mix, sidechain and master processing.

How to verify:

1. Keyboard: make kick and snare duck a bass pad. Adjust attack and release. Check the sub path.
2. Enable master processing. Compare the LUFS meter and peak meter with bypass.
3. Controller: repeat source and target selection, filter changes and master settings.
4. Listen for clear drums and controlled bass, without clipping or unintended stereo collapse.

### Change 108 — `108-tempo-movement`

Goal: move pad filter and pitch in time with the project, as the first part of VISION §16a.

Behavior: tempo-synced LFOs modulate pad filter and pitch.
Delay time follows note divisions, including 1/8, 3/16 and 1/8T.
A project tempo change updates the movement smoothly.
Pad pressure, macros, snapshots and morph are not part of this change.

Spec deltas: `pad-playback`, `audio-engine`, `editing`, future `transport`, `projects`,
`screen-layout`, `screen-help`, `midi-learn` and `command-panel`.

Tasks:

1. Define the initial LFO shapes, rate divisions, depth, phase and restart/free-run rules.
   Define stop, half-time and tempo-change behavior before code.
2. Add pad filter and pitch targets. Keep modulation off in old projects.
3. Add tempo-sync delay divisions. Keep existing free delay times available.
4. Add rate, target and depth controls for both inputs, with visible state and value feedback.

Tests: LFO periods and delay echoes against transport time, triplets, restart and tempo changes.
Check project restart, smoothing, no allocation and callback cost with modulation active.

Planned render: `renders/108-tempo-movement.wav`, filter and pitch LFOs plus each delay division.

How to verify:

1. Keyboard: enable a filter LFO on a bass pad. Select a tempo division. Try pitch modulation.
2. Select 1/8, 3/16 and 1/8T delay. Change project tempo while the pattern plays.
3. Controller: repeat target, rate, depth and delay changes through the parameter pages.
4. Listen for movement and echoes in time, without clicks during tempo changes.

## 4. Later, not in this track

Time stretch without a pitch change, multiband distortion and compression (OTT style),
and comb and formant filters remain later work.
Layers per pad, scenes and song mode also stay outside this track.

Time-stretch candidates, not selected dependencies:

- [Rubber Band](https://breakfastquay.com/rubberband/): GPL, with a separate commercial license option.
- [signalsmith-stretch](https://github.com/Signalsmith-Audio/signalsmith-stretch): MIT.
- WSOLA for drums: an algorithm candidate, not a library or a license choice.

The [aubio license](https://github.com/aubio/aubio/blob/master/COPYING) is GPL-3.0.
It is the proposed detector for 102, not an accepted library decision for that change.
Check exact library versions, wrappers, optional backends and their licenses before adding dependencies.
The project remains GPL-3.0. This plan adds no dependency.
It makes no real-time performance claim for a candidate.
