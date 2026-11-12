# Plan: effects, filters, and pad sound

This plan records the owner's sound and code-structure decisions. It is a guide, not an active requirement.
Requirements come from `openspec/specs/` and the active OpenSpec change. This document creates no sound change.
Implement the changes in section 4 in order. Use 111 for `master-safety` and the next free OpenSpec number for each other change.
Give each change its own branch and PR.
Start the next change only after the owner accepts the current change.

## 1. Current code and limits

This plan uses checked code paths. They describe the current implementation, not the target sound.

- `crates/engine/src/fx.rs` has two shared slots. Each slot contains reverb or delay.
  A kind change fades the old tail. Parameters glide. The engine skips a slot with no input.
- The reverb follows the Freeverb structure. Each side has eight damped combs in parallel and four allpasses in series.
  Both sides receive the sum of the two inputs. Right-side delays are 23 source-rate samples longer.
  `size` sets comb feedback from 0.70 to 0.98. `tone` sets loop damping.
  There is no pre-delay, input low cut, width control, or modulation.
- The delay has two channels and a linearly interpolated read point. Its time glides over about 50 ms.
  That glide changes pitch. Feedback ranges from 0 to 0.9. Maximum free time is 2 s.
  There is no feedback filter, ping-pong mode, or tempo sync.
- Drive uses a Padé approximation of a tanh soft clip. Maximum input gain is ×25.
  The reference peak is 0.3. The dry-to-shaped mix reaches full strength at 25 % amount.
  Drive has no anti-aliasing.
- `crates/engine/src/filter.rs` has a TPT state-variable filter for each voice.
  Its kinds are off, low-pass, high-pass, and band-pass, with a 12 dB/oct slope.
  Maximum Q is about 16. The filter does not self-oscillate.
  Cutoff glides in log frequency over about 10 ms. During a glide, each voice can calculate `tan` each frame.
- `crates/engine/src/lib.rs` applies filter and drive to each voice. It takes sends after voice gain.
  Shared slot returns enter the mix at unity gain. Each slot has no return level or mute.
  Master processing has a gain only. The engine has no limiter or clipper.
  A device can cut values above 1.0. Voice sample reads use linear interpolation.
  Pitch-up aliasing from that read belongs to 110, outside this plan.
- There is no explicit FTZ, DAZ, FPCR FZ, or state-value denormal guard in the engine.
  Reverb, delay, and filter states can decay to denormal values. This may cause some 94a callback peaks. Measurements must test that cause.
- Pad settings include gain, semitone pitch, envelope, filter, group, drive, sends, mode, region, and pitch gate.
  They have no pan or cent fine tune. `PITCH_LIMIT` is an `i8` limit of ±24 semitones.
  Attack, decay, and gate release use linear gain ramps. Voice gain multiplies linear velocity by pad gain.
  Half velocity gives about −6 dB relative to full velocity.
  A pad cannot play its region backward. Plan 114 has a separate reverse sample operation.
- The metronome click enters the left and right mix in `mix_span`, before master gain and the output meter.
  Thus master gain changes the click, and the current meter includes it.
  CPAL output recording takes that mix. JACK output recording takes a device monitor, which includes the click when it monitors ptsr playback.
  Plans 105 and 115 specify internal tempo resampling and offline export. Their click paths do not exist yet.
- Sample loading uses the `rubato` FFT resampler. Keep this path.

The file sizes at planning time are about 22,145 lines in `crates/app/src/main.rs`, 11,613 in
`crates/tui/src/lib.rs`, and 12,755 in `crates/app/src/input_tests.rs`.

## 2. Rules for each sound change

- Follow the real-time rules in `AGENTS.md`. Do not allocate, lock, log, perform I/O, or panic in the callback.
  Allocate buffers with the engine. Replace prepared structures with an `Arc` swap and retire them outside the callback.
- Glide each parameter change. Crossfade each kind or read-point change. Check for unwanted clicks.
- Match effect amount to the reference sound within ±1.5 dB RMS, as required for 113.
  Define the reference and measurement window in each design. Intentional envelope and echo decays have separate targets.
- For each sound change, add WAV files to `renders/` and automatic checks for finite samples and clipping.
  Check expected silence and timing within ±1 sample where timing applies. Use generated sound or permitted CC0 recordings.
- Measure callback cost before and after each change. Include the result in its proposal and owner report.
  State the test rate, buffer size, active voices, and effect path. Do not claim that p99 alone closes the 94a budget.
- When saved settings change, add a session-format migration and tests for old projects.
  State how their sound changes. Keep old projects close to their old sound.
- Give each new parameter a keyboard path, a MIDI learn name, and a position on the FX screen or a pad edit page.
  Apply `VISUAL.md` to value pop-ups, badges, and hints. Test keyboard and raw MIDI paths.
- Each proposal gives separate keyboard and controller checks, exact owner listening steps, and the WAV names.
  Hardware-dependent results remain unverified until the owner checks the device.

## 3. Code-structure rule and cleanup

Add the file-growth and change-number rules to `AGENTS.md` in this planning PR.
CI must compare `crates/app/src/main.rs` and `crates/tui/src/lib.rs` with the PR base.
CI fails if either file grows, unless the owner approves an explicit exception for that PR.
New code goes into modules. Each new behavior change has one OpenSpec number and one PR.
Do not combine new work or use a letter suffix unless the owner asks for that exception in the PR.

`split-modules` is a behavior-neutral OpenSpec change with its own next free number, branch and PR.
It can run between two sound changes, with no other active change.
Prefer to complete it before `pad-bus`. Split `crates/tui/src/lib.rs` by screen: pads, chop, files, FX,
pattern, settings, and help. Put badges, status line, pad grid, and wave drawing in shared widgets.
Move each screen's app logic from `crates/app/src/main.rs` into its module, near existing modules such as
`chop.rs`, `pads.rs`, and `screens.rs`. Split `crates/app/src/input_tests.rs` by topic into
`crates/app/src/input_tests/`. Make each move a separate commit. Do not edit test bodies during a move.
All tests must remain green. Check that behavior, keys, saved data, and sound do not change.

## 4. Ordered changes

### 4.1 `denormals`

Prevent denormal slowdown and measure its contribution to 94a.
At the start of each audio callback and offline render, set MXCSR FTZ and DAZ on x86_64.
Set FPCR FZ on aarch64. The audio thread owns this state, so do not restore it.
Use one small function. Test that a denormal input becomes zero.
As a second guard for platforms without the flag, snap reverb comb damping states and filter integrators
below 1e-20 to zero once per block.

Render 2 s of noise through reverb and delay at maximum feedback, then 60 s of silence.
Silent-part block times must stay within 20 % of active-part times.
Run `task bench` and compare 84b overrun records before and after.
Record which 94a cases improve and which remain. Do not close 94a until all criteria pass.

### 4.2 `metronome-path`

Give the click a separate buffer. Add it to device output after the master stage and the `master-safety` limiter.
Before that limiter exists, keep the click separate from the master path and define the final order in the design.
Give the click its own 0–100 % level setting and knob path. Keep room for a later 1.x cue output.
The click must not enter master meters, output recording, internal resampling, or export.
For JACK, define how `Ctrl+T` excludes the click when it records a device monitor that can contain other applications.
Do not claim a recording test passes until the change implements and measures that routing.
Test click correlation in output recording, a silent master meter with click on, and isolation of the click level setting.
Test internal resampling and export when those paths exist.

### 4.3 `fx-cost`

Make filter and effect cost lower and steadier. During cutoff glides, calculate filter coefficients once per 16 frames and interpolate them.
Alternatively, use a `tan` approximation with at most 0.1 % error up to 0.45 × sample rate.
Set a sweep-render difference limit in the design. Calculate `Drive::new` once per 16 frames during its glide.
Replace per-frame `%` wrap in delay and reverb with compare and reset.
Benchmark before and after with all 32 voices gliding their filters.

### 4.4 `master-safety` (planned 111)

Add an always-on master limiter with a −0.3 dBFS sample ceiling, 1–2 ms look-ahead,
and about 50–100 ms release. A normal mix must not pump audibly.
Measure 4× oversampled inter-sample peaks against a limit set in the design.
Define the 0 dB reference and headroom for 32 voices and both shared slots.
Add a 0–100 % return level per slot, default 100 %, and a slot mute on the FX screen.
Show a short limiter mark when gain reduction exceeds 1 dB.
Render a loud drive-and-reverb pattern and record its limiter gain reduction.
Keep the click outside this master stage, as specified in `metronome-path`.

### 4.5 `pad-sound`

Add four pad controls in one change. Save each in the session, give each a keyboard path and MIDI learn name,
and show each on a pad edit page. Migrate old pads to values that preserve their old sound.

- **Pan:** −100 to +100 % per pad. Use a constant-power law and a centre reference that keeps today's centred mono level.
  State the exact law in the design, including the requested −3 dB centre convention.
  For stereo samples, pan acts as balance. Glide changes without clicks. Take sends after pan.
- **Fine tune:** −100 to +100 cents, added to semitone pitch.
  Shift and the held controller modifier select a fine step. Show both values, for example `+3 st −12 ct`.
- **Envelope curve:** keep linear or select exponential decay and gate release.
  Decide whether attack stays linear in the design. Optionally add 0–2000 ms hold between attack and decay.
  At the set decay time, exponential gain reaches the defined end level, for example −60 dB, then fades to zero.
- **Velocity curve:** set a pad range in dB, with 0 dB making all hits equal.
  Set 30 dB as the proposed new-pad default. Velocity 127 reaches pad gain and velocity 1 reaches that gain minus the range.
  Use a curve linear in dB. Old pads retain their linear velocity and linear envelope behavior exactly.
  State the migration and its effect on old and new pads in the design.

Render a pan sweep and ±100-cent tune against a reference tone.
Render linear and exponential decays of a long sample. Render a velocity ramp from 1 to 127 at three ranges.
Check the specified levels and endpoints.

### 4.6 `pad-reverse`

Add a per-pad forward or reverse play direction. Reverse reads the saved region from end to start.
Keep its envelope, pitch, filter, and fades. Support one-shot and gate modes.
Support loop mode when `pad-loop` exists, with its own loop and crossfade tests.
Do not change sample data. Expose direction as a later 0.4b step-lock target.
Render a region in both directions and a reverse start fade. Check that the first reverse frame does not click.

### 4.7 `pad-bus`

Sum each pad's voices into a pad bus from a fixed pool, for example 32 buses.
Assign a bus only while that pad sounds or has an insert tail.
Keep the filter per voice, including its later envelope. Move drive to the pad bus as an insert.
Take sends after bus inserts. Release a bus after its tail stays below a threshold for a short time.
Overlapping hits on one pad now distort together. Specify and render the difference from the old sound.
Prepare the orderable four-effect insert chain in 113, but add no new effect in this change.

### 4.8 `drive-aa`

Add anti-aliasing with first-order ADAA or 2× oversampling and a half-band filter.
Choose after measuring alias level and CPU cost.
At 48 kHz, render a 5 kHz sine with full drive. Set and test an alias limit, with −60 dB
relative to the fundamental as the proposed limit. Check the ±1.5 dB RMS amount target.

### 4.9 `filter-v2`

Keep 12 dB/oct and add 24 dB/oct with two SVF stages in series.
Put resonance on only one stage. Add notch and peak from the same SVF outputs.
Crossfade kind changes. Add a per-pad filter envelope with attack, decay, amount −100 to +100 %,
and velocity amount 0–100 %. Each voice starts its envelope on a hit.
Define the octave span of envelope amount in the design. Add 0–100 % cutoff key tracking by pad pitch.
The in-loop filter drive is optional. Default: omit it from this change and add a backlog row.
Render each kind and slope in a sweep, a bass envelope, and resonance level comparisons.

### 4.10 `reverb-v2`

Replace Freeverb with an eight-line feedback delay network.
Use mutually prime line lengths and a Hadamard or Householder feedback matrix.
Filter damping in each line and modulate lengths slowly and slightly to reduce metallic ringing on drums.
Diffuse the input through four short allpasses.
Provide RT60 from 0.2 to 20 s, tone, and pre-delay from 0 to 200 ms with optional tempo sync.
Provide input low cut from 20 to 500 Hz, default 150 Hz, and width from 0 to 100 %.
Keep size and tone on the main FX page. Put other controls on a second page.
Measure RT60 within ±10 % and echo density by 50 ms.
Keep tail spectral peaks and full-width left-right tail correlation below limits set in the design.
Repeat the denormal cost check. Migrate old size and tone to a close-sounding decay and tone.

### 4.11 `delay-v2`

Add feedback low-pass and high-pass filters. Each repeat gets darker and thinner without low-end buildup.
Support stereo and ping-pong modes. Support a stereo right offset from 0 to 20 ms.
Add free milliseconds and tempo sync from 1/32 to 1/1, including dotted and triplet values.
Allocate enough memory for the longest synced time at 40 BPM.
Provide glide, the current pitch-slide behavior, and jump, a crossfade between read points.
Default to glide. Optional loop saturation uses a level-matched tape amount.
At 172 BPM, check repeat times within ±1 sample, progressive darkening, alternating ping-pong sides,
and the end of a tail at 90 % feedback.

## 5. Follow-on plans and decisions

When these changes finish, update `V1.md` with measured results, not planned completion.
Mark 111 complete only after the owner accepts `master-safety`. Record the 94a result from `denormals`.
Make 113 build on `pad-bus`. Account for delay sync and drive completed here.
Keep voice sample interpolation and pitch-up quality in 110.
Add one `docs/BACKLOG.md` row for each open item left by this work, including in-loop filter drive if omitted.

The defaults below are design inputs. Record each final owner decision in its active change before code.

| Before | Question | Default |
|---|---|---|
| `pad-sound` | Pan law | Constant power, −3 dB at centre, with today's centred mono level preserved. |
| `pad-sound` | New-pad velocity range | 30 dB. |
| `pad-sound` | New-pad envelope curve | Exponential decay. |
| `pad-sound` | Old pad migration | Keep linear velocity and decay exactly. |
| `pad-bus` | Distort overlapping hits together | Yes. |
| `filter-v2` | Drive inside the filter loop | No in this change. Put it in the backlog. |
| `reverb-v2` | Default input low cut | 150 Hz. |
| `delay-v2` | Default time-change mode | Glide. |
| Code structure | CI rejects growth of the two large files | Yes. The owner can approve an exception in a PR. |

The plan moves part of the former 0.3 and 113 scope forward by the owner's request.
`pad-bus` means one bus per sounding pad. The later 1.x bus item means groups of pads with separate chains.
