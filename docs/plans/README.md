# Plans: controller-first work, patterns and sample workflows

## Start here: v1.0.0

[V1.md](V1.md) is the release plan: sample → chop → pads → pattern → resample → effects → export.
It selects the minimum for both keyboard and SMC-PAD, with no clicks and no crashes.
Its scope and milestone order take priority over the broader tracks below and the historical vision stages.
Everything outside that minimum is `1.x`. Only the owner can move work into 1.0.
The quality, export and release changes retain numbers 110–120. Deferred callback investigation keeps 94a. Change 109 is canceled.
The new glitch-kit and pad-sound names are planning labels. Assign numbers before their OpenSpec proposals.
Independent report-only change 84b adds local overrun records before further product work.

The functional order is input/controller (0.2), patterns (0.4), glitch kit (0.4b),
tempo/resample/effects/export (0.5), then data safety and release (0.6).
The owner deferred sound-quality work (0.3). The project now includes transport 95 and patterns 96–98k.
On 2026-11-12, the owner chose a full-bank A1 overview before targeted step editing.
Change 98l adds the overview. Change 98a adds one-hit correction. The earlier 98a draft is a historical guide.
Edit panels share the pad edit frame since 98d–98h.
Existing real-time rules, tests and benchmark limits stay mandatory. Input prerequisites need explicit decisions before code.
The owner expanded the 1.0 sound scope on 2026-11-07:

- **0.4:** quantize and step grids include 1/64. The default step view stays 1/16.
- **0.4b, in order:** minimum step locks, ratchets, held master stutter, roll matrix, then offline PaulStretch.
  Held performance uses start on the next 1/16 boundary, with 1/32 as another setting.
  The pattern runs underneath with its normal output silent. Release returns on the next hit, without unintended gaps or doubled hits.
  The latched `ROLL` layer provides 16 held cells on keyboard and SMC-PAD, with rate/variant glyphs and a badge.
  A keyboard without releases plays a cell for one bar. Each cell gets a checked render at 172 BPM.
- **0.5:** retain shared reverb/delay sends and add serial insert chains with up to four effects per pad and master.
  Include drive/waveshaper, bit/rate crusher, filter and compressor. The owner can change their order.
  A soft/hard master clipper defaults to off before the always-on limiter.
  Add pad pitch envelopes and crossfade loop playback for gate and one-shot pads.
  Add makeup curves with RMS checks at 0/50/100 %, within ±1.5 dB, a one-knob master DJ filter and a rate/bit lo-fi insert.
- **Release checks:** the owner listens during normal use and reports unwanted clicks with a reproducible scenario.
  Keep existing mute groups and assignment defaults. The owner removed universal fades and automatic slice grouping from the plan.
  Change 96 plans the pattern-step pitch gate. It defaults to on for slices.
  The callback workload includes full insert chains and simultaneous 1/32 ratchets at 200 BPM on 16 pads.
  Three finished tracks cover glitch ambient, drum and bass or jungle, and industrial hardcore or hard techno.
  Each friction item names its source track.

The exact ranges, real-time rules and 1.x boundary are in [V1.md](V1.md).
More locks from 104, phrase variation, real-time old-sampler stretch, tuned comb feedback and MIDI clock in/out remain `1.x`.
Slice follow, granular/spectral tools, buses, sidechain and chromatic/LFO work also remain `1.x`.
Phrase variation replaces mutations and never rewrites the stored pattern.
Until real-time stretch exists, the 1.0 matrix uses half speed, one octave down.
See [V1 owner questions](V1.md#6-owner-questions) for the hardware-dependent work order.
The [friction log](../V1-FRICTION.md) starts with current-program sessions. It has no recorded sessions yet.
These documents do not activate a change or claim success on a release criterion.

## Broader planning context

These plans were imported from the owner's eight planning documents on 2026-11-03.
The vision is in `../VISION.md`. This folder also holds later plans for project safety
and sample workflows. Describe implementation tasks without specific genre names.
Release acceptance uses the three styles that the owner selected in V1.
These plans guide the agent, like `docs/VISION.md`. They are not requirements.
Requirements come only from `openspec/specs/` and the active change.
Each change starts with an OpenSpec proposal, design, spec deltas and tasks.
Follow `AGENTS.md`: one change at a time, specifications before code, a test for each scenario, atomic commits and a successful `task check`.
A change normally has its own branch. Earlier combined branches were explicit owner exceptions,
not permission to combine the new changes.

## Current status (2026-11-08)

- Change 82 was merged in [PR #3](https://github.com/fl64/ptsr/pull/3). The owner accepted
  all remaining checks. It is archived in
  `openspec/changes/archive/2026-11-03-82-shared-fx-screen`.
- The owner requested `plan/controller-first-patterns` for 84, 84a and 85 as a branch exception.
  Closure of 85 and work on 86–86a later shared `86-screen-registry` and
  [PR #5](https://github.com/fl64/ptsr/pull/5). The owner accepted, archived and merged 85–86a on 2026-11-04.
  Squash commit: `5ef415f8`. New implementation changes use separate branches.
  Push and PR need a separate request.
- The owner accepted keymap cleanup 83a. Its archive commit is `2b084d1`.
  Archive: `openspec/changes/archive/2026-11-03-83a-keymap-cleanup/`.
  The shipped map has Alt+R mic, Ctrl+G normalize and H / Shift+H hits.
  U/O change banks. J/L change values. Tab changes assignment order. Ctrl+C copies a pad.
  The owner accepted without completing the manual checks in section 83a of `docs/LIVE-CHECKS.md`.
  This acceptance does not imply a physical SMC-PAD or listening pass.
- The owner checked and accepted input parity 84 on 2026-11-03. It is archived in
  `openspec/changes/archive/2026-11-03-84-input-parity/`, based on `2b084d1`. Its paired
  input tests cover the approved routing and release fixes, the Chop pad playback
  line, fixed LISTEN and F2 PLAY badges. Physical-controller checks remain unverified.
  Callback benchmark peaks above the 50% budget target remain unresolved.
- The owner accepted report-only change 84a on 2026-11-03. It is archived in
  `openspec/changes/archive/2026-11-03-84a-benchmark-report/`.
  It was inserted before 85 after the benchmark investigation. Wall and thread CPU times,
  estimated gaps, exceedance counts and empty-call overhead are now reported. The sound,
  workload, 50% target and existing wall p99 exit checks stay unchanged. Scheduler delays
  were observed, but slow-core CPU spikes remain open. No hardware pass is implied.
- The owner accepted and archived report-only change 84b for automatic local overrun records.
  Archive: `openspec/changes/archive/2026-11-05-84b-benchmark-attribution/`.
  It follows the 84a report, without changing sound processing, workload or exit gates.
  Generic code, tests, instructions and anonymized verification summaries can enter Git.
  Raw CSV files, detailed machine logs and identifying local paths stay local.
  This diagnostic does not close the callback maximum-budget gate or activate 87.
- The owner accepted input modes 85 on 2026-11-03. It is archived in
  `openspec/changes/archive/2026-11-03-85-input-modes/`. Implementation and offline checks are complete.
  It shares
  pad handling and MIDI state, fixes controller-command order and deferred releases,
  and separately fixes mixed Chop holds and late direct notifications. The dispatch
  move is behavior-neutral. Keys, profiles, saved data and the callback stay unchanged.
  The three 85 WAV files and separate owner passes are in section 85 of
  `docs/LIVE-CHECKS.md`. This is overall acceptance, not individual listening results.
  Physical-controller checks remain unverified, and separate WAV listening remains unreported.
- The owner accepted the screen registry (86) on 2026-11-04.
  Its archive is `openspec/changes/archive/2026-11-04-86-screen-registry/` on `86-screen-registry`.
  Implementation is complete at `a186fbb`. Commit `e90543c` records offline verification.
  The registry keeps existing screen identity, help, bindings and FX rules for valid input.
  Invalid shared MIDI state has a separate, specified correction in `a186fbb`.
  It rejects new actions without losing original releases or pending-input order.
  Section 86 of `docs/LIVE-CHECKS.md` records evidence and separate keyboard, controller and listening checks.
  This is overall acceptance, not individual keyboard, physical-controller or WAV listening results.
  Earlier callback peaks and slow-core CPU spikes remain unresolved. That acceptance did not activate 87.
- The owner accepted and archived legacy folder migration removal (86a) in merged PR #5.
  Archive: `openspec/changes/archive/2026-11-04-86a-remove-legacy-migration/`.
- The owner accepted and archived save retry (86b), safe project transitions (86c) and portable chops (86d) without the manual passes.
  Archives: `openspec/changes/archive/2026-11-05-86b-save-retry/`,
  `openspec/changes/archive/2026-11-05-86c-safe-project-switch/` and
  `openspec/changes/archive/2026-11-05-86d-portable-chops/`.
  Keyboard, physical-controller and listening results remain unverified. Callback maxima above 50% remain unresolved.
  The save-safety sequence is complete. 84b and 86b–86d merged in [PR #9](https://github.com/fl64/ptsr/pull/9) on 2026-11-05.
  That combined PR was an explicit exception to separate change branches.
- The owner requested `87-status-badges` on 2026-11-05 and approved its specification step and local signed-off commits.
  The owner accepted 87 on 2026-11-07 and authorized archive, checks and squash merge in [PR #10](https://github.com/fl64/ptsr/pull/10).
  Archive: `openspec/changes/archive/2026-11-07-87-status-badges/`.
  It covers existing mode badges, pad marks, notices and F1 Messages, values, input status, questions and an offline screen catalogue.
  The owner approved the specification defaults and the first implementation file plan.
  The behavior-neutral preparation is complete: 12 screen snapshot methods are in `crates/app/src/view.rs`.
  Shared title badges now identify existing modes, with compact edit-page names and compound placement labels.
  Settings keeps its title visible when MIDI rows scroll. PLAY and LISTEN keep their widths. Source-specific input priorities stay unchanged.
  Pad corners now show placement targets, existing replacement questions and planned slice numbers. Yellow mode frames retain pad state and bank identity.
  Runtime notices share levels, visible-time lifetimes, priority and latest-50 history.
  F1 opens Help first. Tab/Shift+Tab selects Messages, with levels, origins and wrapped text.
  Up/Down and PageUp/PageDown scroll Messages. F1/Esc closes help without changing the work through those local controls.
  Each result keeps its keyboard, actual device or system origin. Startup acknowledgement and silent successful autosave stay unchanged.
  A new press acknowledges only the Error that a frame showed. Releases and repeated knob values do not acknowledge it.
  Offline notice and save-priority regressions pass. The owner accepted step 4.3.
  F1 navigation retains parameter pages and banks behind help. Existing pad routes and higher-priority input remain unchanged.
  Help and Messages share the owner-accepted stable frame.
  Applied values now use a shared one-second popup with object labels and numeric range bars.
  Pickup, refusals, scope-limit messages and editor priority retain their behavior.
  Input-source state now keeps connection identities and original receipt times across production routes.
  Old connections cannot activate a new connection. Delayed delivery does not extend the original deadline.
  Project changes keep source identities and deadlines.
  Source feedback does not change actions, acknowledgements, velocities or release ownership.
  Shared status keeps recording seconds, permanent KB, the last used controller number, actual audio data, master and project details.
  Input emphasis lasts one second. Keyboard activity and expiry retain the controller number and fixed columns.
  A permanent `*` is dim when saved and yellow when pending. `Save failed` remains explicit.
  Full port names stay in F12 Input. CPU and full xrun diagnostics stay in F12 Audio.
  The incomplete SMC-PAD example uses `.toml.example` and is not a working profile.
  Screen learning uses the fixed three-row help band and keeps the status visible. Its hints follow effective keys.
  Unchanged badge, input-label and status text now use UI-thread caches. Timers, meters, notices, playback and hints retain live updates.
  Questions show effective keyboard and connected MIDI answers. Large questions retain safety text and answer rows without changing target checks.
  Text overlays now remove intersecting kitty pictures before text output and restore the current wave afterward. Unoccluded pictures stay unchanged.
  `task screens` now checks the screen/mode matrix and writes deterministic ANSI files at both supported sizes.
  The files stay local under `renders/screens/`. Real kitty composition still needs a manual check.
  Active-delta coverage now checks test declarations and their attached scenario descriptions.
  Documentation now links catalogue files and separate keyboard, controller, graphics and listening procedures.
  Overall acceptance does not establish individual keyboard, physical-controller, kitty or listening results. Unreported checks remain unverified.
  Earlier callback maximum-budget failures remain unresolved.
  Sound, factory bindings and saved formats stay unchanged. Generic controller confirmation remains in 88.
  Closure order: owner acceptance, archive in the same PR, checks, then squash merge. Keep the branch after merge.
- The owner accepted `87a-copy-names` on 2026-11-07 after the short keyboard check.
  Archive: `openspec/changes/archive/2026-11-07-87a-copy-names/`. The main pad-assignment specification contains its requirements.
  Copies use one `(copy N)` marker and the smallest free number across all current banks.
  Manual canonical markers have the same meaning. The target's current name stays occupied until replacement succeeds.
  Existing names, sample files, sound, controls and saved formats stay unchanged.
  The owner reported success for repeated copies, a copy of a copy, numbered names and silent placement.
  Remaining keyboard checks, physical-controller checks, separate listening and callback maximum-budget limits remain open.
  The owner authorized archive, checks, signed-off commits, push, a PR and squash merge. Keep the branch.
  Work stays in the main repository folder. Change 88 is not active.
- The owner deferred 88 on 2026-11-07 until a MIDI device is available for live checks.
  Do not start it now. Recorded MIDI tests cannot establish the physical-controller result.
  Later input prerequisites remain.
- Transport 95 is archived and merged in [PR #13](https://github.com/fl64/ptsr/pull/13).
  It implements the audio-frame clock, project tempo, metronome, shared controls and visible clock state.
  Format 4 saves tempo and metronome preference, not running position. Patterns are not implemented.
  Individual keyboard, physical-controller and listening results remain unverified.
  Start 96 from this implementation, not from the older 95 sketch in the imported plans.
- The owner deferred separate sound-quality work until a separate request and chose pattern playback next.
  Callback investigation 94a is no longer a separate prerequisite before transport or patterns.
  Historical callback maximum-budget failures remain unresolved. Existing checks and benchmark limits stay mandatory.
  Diagnosis of the reported Chop-start click remains deferred. Its cause is unknown.
  The owner removed the universal fade requirement. Keep mute groups off / 1–8 and manual assignments unchanged.
- Change 83 is superseded as a plan by 91. Keep its deferred artifacts as source material
  until 91 carries the useful parts and removes the old folder.
- The SMC-PAD port, preset, relative encoding and lights still need measurements on the
  owner's Linux machine. Reference maps are not hardware acceptance.

## The goal of the owner

The goal is **every action that does not need typed text** from the M-VAVE SMC-PAD alone after startup.
This includes loading, chopping, assignment, pad shaping, effects, projects and settings. Later, it includes pattern recording and playback.
These operations must be fast, clear and need few presses.
Typed text stays on the keyboard: pad names, new project names and the file filter.

Measurable form of the goal:

- Every action in the program that does not need text has a controller path.
- A frequent action takes 1 press. Any other action takes not more than 2 presses
  (for example: hold STOP + a pad) from any screen, plus knob turns for values.
- The keyboard and the controller do the same thing for the same logical action, on every
  screen and in every mode. A test proves this.

## Project save safety before 87

These three small changes record the save review. They are plans, not requirements.
Create their OpenSpec artifacts before code. Implement and check one change at a time.
They keep keys, saved formats and sound processing unchanged.

### Change 86b — `86b-save-retry`

Goal: retain unsaved work after a write failure and permit a manual retry.

Behavior: clear the unsaved state only after a successful write.
Distinguish a retryable write failure from protection after a read error.
Each Ctrl+S reports success or a reason for refusal.
After a write failure, pause automatic retries until a manual save succeeds.
Then resume normal autosave. Keep a persistent `Unsaved` or `Save failed` indicator without a new input mode.
Never overwrite a damaged or newer-format file with an empty replacement.
Never recreate a removed project folder during a save.

Spec deltas: `pad-assignment`, `projects`, `screen-help` and `screen-layout`.
Tasks: separate the save states, retain the last saved file through temporary-file replacement,
add retry feedback and test the indicator from both input paths.

Tests: injected write refusal, repeated manual refusal, successful retry and later autosave.
Check the last saved file byte for byte, read protection and a removed folder.
Use controlled time, not sleeps. Use an isolated failure fixture, not a full disk.

How to verify: use a disposable project. Change a pad, force a write refusal and press Ctrl+S twice.
Restore write access. Save again. Check the indicator and the result after another edit.
Repeat the save action through a learned controller control. This plan adds no new sound render.

### Change 86c — `86c-safe-project-switch`

Goal: keep pending work when the owner selects or creates a project.

Behavior: save dirty project or scratch work before a transition.
After success, open the target without another question.
On refusal, keep the current pads, chops, undo, unsaved state and picker selection or entered name.
Selecting the open project's folder closes the picker without reloading it.
A target read failure keeps the current project and last-project setting unchanged.
Startup read-error behavior stays unchanged.
A clean scratch session with read protection can be left without writing its file.
Unsaved work still blocks a transition when a safe save is not possible.

Spec deltas: `projects`, `pad-assignment` and `screen-help`.
Tasks: use the save result from 86b for open and create. Guard same-folder selection.
Apply target state only after validation and a successful required save.

Tests: switch before autosave, scratch-to-project, a new project and the same project selected again.
Inject source write failure and target read failure. Check pads, chops, undo and the picker.
Use controlled time and production keyboard/MIDI dispatch. Keep the existing sound-isolation render tests.

How to verify: edit and switch at once, then return. Repeat through the controller picker.
Select the open project again. In a disposable project, refuse a save and try to switch.
The work and undo stay available. This change adds no new sound algorithm.

### Change 86d — `86d-portable-chops`

Goal: keep every pad-linked chop variant after a project folder moves.

Behavior: apply the same stored/resolved path rules to pads, current chops and `kept` variants.
Internal paths are relative. External paths stay absolute. Do not copy samples.
Keep cuts, links, epochs and pad parameters. Old projects without `kept` retain their existing fallback.
Do not guess new locations for stale absolute paths in projects that already moved.

Spec deltas: `projects` and `chopping`.
Tasks: convert all variant keys on save and load, without a format change.
Test through the production project loaders, not only the path helper.

Tests: save two pad-linked variants, with extra cuts that have no pad. Move the project folder.
Open both variants through F3. Check all cuts, links and parameters, then save and open again.
Check external files and older projects. Render the restored slices with the normal automatic checks.

How to verify: repeat the two-variant move with a disposable project from each input path.
Open a pad from each variant and move a cut. Only that variant's linked pads follow it.
Local render: `renders/86d-portable-chops.wav`. Compare matching before/after slices as section 86d of `docs/LIVE-CHECKS.md` describes.

## Broader tracks and visual rules

This is the full roadmap, not the 1.0 feature list. V1 selects reduced scopes from it.
Unselected parts remain `1.x`, even when they have a planned change number.

| Order | Track | Document | Changes |
|---|---|---|---|
| 0 | Consistent keys and reserved recording / redo chords | `KEYMAP-CLEANUP.md` | 83a |
| before 87 | Save retry, safe project switches and portable chop variants | This document, Project save safety | 86b–86d |
| 1 | Input parity, refactors and new input features. Change 87 is the visual base | `REFACTOR-INPUT.md`, `VISUAL.md` (87) | 84–90 |
| 2 | SMC-PAD integration and the command panel | `SMC-PAD-INTEGRATION.md` | 91–94 |
| deferred | Investigate callback peaks and slow-core CPU spikes after a separate owner request | `PATTERNS.md` | 94a |
| 3 | Transport, patterns, recording, sequencer | `PATTERNS.md` | 95–100a, optional 101 |
| 4, after 100a | Sample tempo, break patterns, locks, resampling, bass, mix and movement | `SAMPLE-WORKFLOWS.md` | 102–108 |
| all | Tools, safety, comfort, docs and risks. Changes 93a and 100a | `CROSS-CUTTING.md` | tasks in many changes |

`VISUAL.md` also gives rules for every later change.
Each new mode or state brings its badge, pad glyphs, notices and hints for both inputs.

The order is not free. Track 2 needs the mode model, the button hold and the relative encoders
of track 1. Track 3 needs the screen registry for a new screen, plus `ui.confirm` and button holds (`erase`, note repeat).
It also needs the track 2 command panel for the new actions' controller paths.
Project save safety 86b–86d comes before 87. The owner deferred separate callback investigation 94a, not existing benchmark checks.
Track 4 follows the pattern base through 100a. It does not wait for optional timestamps (101).
For 1.0, V1 inserts the glitch kit after patterns and before sample tempo, resampling, effects and export.
The minimum locks from 104 move into `step-locks` in 0.4b. The rest of 104 stays `1.x`.
The diagram below retains the broader roadmap, not the revised 1.0 order.
Change 103 needs 102. Changes 104 and 105 need patterns. Changes 106–108 can change order between them.

```
83a keymap cleanup (A confirmed; specifications first)
    │
    ▼
84 parity ─► 85 modes ─► 86 screens
                             │
            86b retry ─► 86c safe switch ─► 86d portable chops
                                                    │
87 badges ◄────────────────────────────────────────┘
    │
88 confirm ─► 89 hold ─► 90 relative ─► 91 smc profile
                                           │
            92 command panel ◄─────────────┘
                 │
            93 controller reach ─► 93a comfort ─┬─► (94 pad lights, optional)
                                               │
            95 transport ◄─────────────────────┘
                 │
            96 pattern play ─► 97 pattern record ─► 98 step + screen
                                                        │
                                  99 pattern slots ◄────┘
                                       │
                                  100 note repeat ─► 100a pad mute
                                                        │
                      ┌─────────────────────────────────┤
                      ▼                                 ├─► 104 locks
            102 sample tempo ─► 103 break to pattern     ├─► 105 resample
                                                        ├─► 106 bass / 107 mix / 108 movement
                                                        └─► (101 input timestamps, optional)
```

In parallel with 84, start step 0 of change 91 without code.
The owner measures the SMC-PAD on Linux and sets the encoders to relative. This removes the biggest risk early.

Change 82 is archived. The new prerequisite for 84 is keymap cleanup 83a.
Change 91 will replace change 83 (`openspec/deferred/83-controller-profiles`).
In 91, move its useful sources, pad orientation and raw-byte test approach into the new change.
Then remove the deferred folder. The diagram retains the controller-first dependency plan, not completed hardware acceptance.
Transport 95 uses existing keyboard and learnable MIDI paths. Physical-controller checks and the unfinished controller work remain open.
Each later proposal must resolve the input prerequisites it needs. Recording still needs an explicit redo decision.
Change 94 is optional. Change 94a waits for a separate owner request, not a separate implementation step before patterns.
A passing p99 check alone does not resolve the historical maximum-budget failures.

Letters keep the existing change numbers valid. These additions do not renumber 87–101.
The owner can change the order, subject to the dependencies above.

## Review notes before implementation

These points qualify the detailed plans. Resolve each in that change's design and spec
before code. The imported plans do not override an existing requirement.

- **Count presses consistently (93):** STOP + a cell is two physical presses. Object and
  target selection and value turns are separate from the command count. Count each command,
  not a whole multi-action story. Test the path from each screen and mode.
  Do not assume an unchanging panel row proves that a command is reachable inside a picker or a question.
- **Protect the latched panel too (92):** PAD BANK layer 2 needs no held STOP. A destructive
  cell must ask a question before it removes or replaces anything. Proposed safe default:
  destructive cells ask in both panel forms. Keep X-Session direct defaults unchanged.
- **No hidden bug fixes (85):** the pending-action acknowledgement changes a race outcome.
  List it as an explicit fix with a scenario, not as a behavior-neutral code move. Resolve
  priority differences found by 84 before the refactor.
- **Preserve hit velocity (84):** the current UI `PadHit` event has no velocity. Carry the
  original velocity when Chop is routed to the UI. Do not replace it with a constant.
- **Keep hidden hits hidden (96):** filter raw hits outside the shortened length before
  quantization. Wrap a quantized hit at the loop boundary only after that filter. Otherwise
  shortening the pattern would move hidden hits into its beginning.
- **Controller-only work needs no fake keyboard equivalent:** test panel holds and layer
  marks with timed MIDI events. Paired tests cover shared logical actions.
  Raw MIDI velocity and device identity have their own tests.

## Rules that apply to every change in these plans

1. **No behavior change hides inside a refactor.** Changes 85, 86 and 90 (model part) keep all
   existing scenarios green without edits. If a refactor shows a bug, stop, write it down, and fix
   it in a separate small change or in 84.
2. **Two inputs, one test.** From change 84 on, each new input scenario runs twice.
   One run uses keyboard events. The other uses raw MIDI bytes through the real profile.
   The harness from 84 gives this.
3. **From change 92 on, each new action states its controller path** in its spec: button, panel cell or knob.
   The action also appears in the reach table of change 93. A new action without a controller path
   fails the reach test, unless it is on the text-only list.
4. **Real time stays safe.** No allocation, locks, I/O or logs in the audio callback. New engine
   data goes as `Arc` through `rtrb`, and old data goes back through the garbage queue.
5. **Each sound change has a render** in `renders/` with automatic checks for finite samples, clipping, level, timing ±1 sample and clicks.
   Check fx-core reference RMS at 0/50/100 % amount against the ±1.5 dB limit.
   Render all roll cells at 172 BPM and the three press/release cases in V1.
   Use generated sounds or the owner's CC0 recordings. Only optional external demo tests can skip for a missing owner-supplied file.
6. **The keyboard keeps its comfort.** The keyboard is the reference.
   Refactors and controller work do not change a key's role, except where the proposal explicitly states a change.
   Every new action also has a keyboard key or path, except controller-only actions.
   Those exceptions are holding STOP for the panel and the layer marks. The reach test of 93
   checks both inputs. Controller-only parts of the screen (the `SMC` help group, the action
   echo, the panel) show only while a controller is connected.
7. **Every new mode or state is visible** as `VISUAL.md` says: badge, pad glyphs, notices with a
   level, hints for both inputs, tui tests.
8. **Owner checks.** Each proposal ends with `How to verify`.
   This section gives exact SMC-PAD and keyboard steps, plus what to listen for.

## Decisions to get from the owner

Ask these before the change that needs them. Write the answer into that change. If there is no
answer, take the default and write the question into `design.md` under "Open questions".

| Before | Question | Default |
|---|---|---|
| 0.2 | Is the SMC-PAD at hand? Without it, 90–93 cannot be accepted | Prepare 96 from implemented transport 95. Resolve input prerequisites in each proposal. Keep sound-quality work deferred and unverified controller checks open |
| 88 | When does confirmation work resume? | After a MIDI device is available for live checks. Owner decision: defer 88 |
| 0.4 | One swing amount in 1.0? | Yes, if 96 makes it cheap |
| 0.4b | Is the glitch kit in 1.0? | Yes |
| 0.4b | Stutter on the master only, or also per pad in 1.0? | Master only |
| 0.4b | Roll matrix target: master, selected pad, or the pattern's last hit? | The last hit of the pattern |
| 0.4b | Effect start grid | 1/16. Setting: 1/16 or 1/32 |
| 1.x | Phrase length for variation | 2 bars |
| 0.5 | Insert slots per pad | 4 |
| 0.5 | Master clipper on by default? | Off. The limiter is always on |
| 0.5 | Export format default | 24-bit WAV at the project rate |
| 0.6 | Release form | One binary in tar.gz on GitHub Releases, built for glibc 2.28 |
| 0.6 | How many autosave versions | 10 |
| 83a | J/L on Pads: values with banks on U/O (A), or keep banks and require edit on FX too (B)? | The owner chose A on 2026-11-03. The archived 83a proposal records the choice |
| 87 | Yellow now means "the pads do something other than sound" (marks, in hand, assign, step, `erase`, panel). Change `docs/ui/STYLE.md`? | The owner approved this meaning on 2026-11-05. Marking still sounds. |
| 92 | Action echo of controller presses in the message line | On, 1.5 s, a setting turns it off |
| 89 | How long is a "hold" for a button? | 300 ms, or at once when another control is used while the button is down |
| 91 | Can the owner set the SMC-PAD encoders to relative mode in MidiSuite once? | Yes. Without it, the encoders stop at the ends of their range |
| 92 | Is "hold STOP" the command panel? | Yes. PAD BANK layer 2 gives the same panel as a latch |
| 92 | May an SMC panel cell apply a layout or answer a replacement question? | Yes, with a visible question for destructive changes in both panel forms. Keep X-Session direct defaults unchanged |
| 93a | Panic on STOP + PLAY together | Yes |
| 93a | Redo on the SMC-PAD: STOP held + RIGHT (undo: STOP held + LEFT) | Yes |
| 93a | A new project from the controller gets a date name (`2026-11-03-1`) | Yes |
| 96 | Does a pattern store absolute pads (0–127) or pads of the open bank? | Absolute |
| 96 | `Ctrl+X` swaps two pads: do the pattern hits follow the sound? | Yes |
| 97 | What is one undo step of recording? | One recording pass, from record on to record off |
| 97 | Stop: do one-shot tails ring out? | Yes. Gate notes get their release |
| 98 | 32 steps per page, or a 1/16 ↔ 1/32 switch with 16 steps? | The switch, with two pages per bar at 1/32 |
| 99 | How many patterns play at the same time before a launcher exists? | One |
| 99 | Default slot switch boundary | 8 bars |
| 102 | Pad fit default for a new slice | Off. The owner enables it |
| 102 | Automatic tempo detection library | aubio (GPL-3.0, compatible). Proposal only. Check the version and wrapper before code |
| 104 (1.x) | Which parameters follow the minimum locks in 0.4b? | Decay and sends, with the exact extension set in its design |
