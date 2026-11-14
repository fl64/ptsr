# Plans: one ordered roadmap

## Start here: v1.0.0

[The ordered roadmap](#ordered-roadmap) sets the sequence for new work. [V1.md](V1.md)
sets the 1.0 release scope and criteria. [FX-FILTERS.md](FX-FILTERS.md),
[UI-UX.md](UI-UX.md), and the other plans give details. An older order in a detailed
plan does not change the roadmap order. Requirements come from OpenSpec, not these plans.

The owner checks each phase by ear and by eye. Each item gets one OpenSpec change,
number, branch, and PR. A phase ends with the owner's listening and controller or
keyboard checks. Do not start the next phase while an item waits for owner checks,
unless the owner allows it. Move an item from Later into a phase only by an owner
decision recorded in the decisions table.

The quality, export, and release changes retain numbers 110–120. Change 109 is
canceled. New names are planning labels. Assign numbers before OpenSpec work.
Existing real-time rules, tests, and callback limits stay mandatory.

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
Slice follow, granular/spectral tools, group buses, sidechain and chromatic/LFO work also remain `1.x`.
The per-pad bus in the FX and filters plan is separate from later group buses.
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
Follow `AGENTS.md`: one change at a time, specifications before code, a test for each scenario, atomic commits, local checks, and green CI.
A change normally has its own branch. Earlier combined branches were explicit owner exceptions,
not permission to combine the new changes.

## Current status (2026-11-14)

The [archives](../../openspec/changes/archive/) hold earlier change history. Keep open manual
checks in [LIVE-CHECKS.md](../LIVE-CHECKS.md). Planned names do not have OpenSpec numbers yet.

| Change | State | Open owner check |
| --- | --- | --- |
| 121 `split-modules` | Archived and merged | None for this refactor. |
| 122 `ui-quiet-words` | Archived and merged | Keyboard, controller, and screen checks remain in `LIVE-CHECKS.md`. |
| 123 `pattern-view-polish` | Accepted and archived in [PR #8](https://github.com/fl64/ptsr/pull/8) | Separate keyboard, controller, screen, and listening results remain open in `LIVE-CHECKS.md`. |
| `repo-fast-checks` | Merged in [PR #15](https://github.com/fl64/ptsr/pull/15). `master` requires Rust for merge. | None for sampler behavior. |
| `denormals` | Planned next | Review the silence render and callback measurements. Keep 94a open. |
| `pattern-screen` | Planned | Check F5, view and edit, the step lane, and controller paths at both sizes. |
| `pads-source-view` | Planned after `pattern-screen`. The plan is in [PR #16](https://github.com/fl64/ptsr/pull/16). | Check voice heads, sample rows, the mini pattern, and pad edit pages at both sizes. |
| `compact-help` | Planned | Check one-row help, full help, and the setting at both sizes. |
| `key-consistency` | Planned | Check bank keys, Files folder-up, and moved-key notices. |

## The goal of the owner

The goal is **every action that does not need typed text** from the M-VAVE SMC-PAD alone after startup.
This includes loading, chopping, assignment, pad shaping, effects, projects and settings. Later, it includes pattern recording and playback.
These operations must be fast, clear and need few presses.
Typed text stays on the keyboard: pad names, new project names and the file filter.

Measurable form of the goal:

- Every action in the program that does not need text has a controller path.
- A frequent action takes 1 press. Any other action takes not more than 2 presses
  from a visible panel, plus knob turns for values. The first panel entry needs
  PAD BANK and two layer 2 hits because the first hit only shows the panel.
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

## Ordered roadmap

Daily use guides this order. An item that the owner can hear or see comes first.
Small internal work stays in [Later](#later), unless it blocks a phase. Each phase
ends with the owner's listening and controller or keyboard checks.

### Phase A — finish and stabilize

1. [123 pattern-view-polish](UI-UX.md#42-pattern-view-polish): accepted and archived.
2. [denormals](FX-FILTERS.md#41-denormals): no slowdown after reverb and delay tails. Report its effect on 94a.
3. [metronome-path](FX-FILTERS.md#42-metronome-path): keep the click out of recordings, resampling, and export. Give it its own level.
4. [master-safety](FX-FILTERS.md#44-master-safety-planned-111): limiter, return level, and mute per FX slot.

The owner hears no clipping, no click in recordings, and no dropouts after long tails.

### Phase B — the sound of a pad

5. [pad-sound](FX-FILTERS.md#45-pad-sound): pan, fine tune, exponential decay with hold, and velocity curve.
6. [pad-reverse](FX-FILTERS.md#46-pad-reverse): play a pad region backward.
7. [pitch-quality](ROADMAP-DETAILS.md#pitch-quality): anti-aliased interpolation for pitched pads.
8. [no-clicks](ROADMAP-DETAILS.md#no-clicks): short fades on starts, stops, steals, cuts, and slice edits.

The owner hears clean pitched breaks, natural decays, real dynamics, and no clicks.

### Phase C — sequencing on screen

9. [pattern-screen](UI-UX.md#31-pattern-screen): F5 screen, view and edit, F7 free, step lane.
10. [control-pages](ROADMAP-DETAILS.md#control-pages): one control table per screen and mode.
11. [pads-source-view](UI-UX.md#32-pads-source-view-includes-pads-context-lane): live source and mini pattern lane.
12. [99 pattern slots](PATTERNS.md#change-99--99-pattern-slots): 16 slots with 4, 8, or 16 bar switches.
13. [100 note repeat](PATTERNS.md#change-100--100-note-repeat): 1/32 and triplets.
14. [100a pad mute](CROSS-CUTTING.md#change-100a--100a-pad-mute-after-100): mute pads during play.

The owner sees and hears a step sequencer, sounding slices, slots, and repeats.

### Phase D — the controller

15. [88 ui.confirm](REFACTOR-INPUT.md#change-88--88-ui-confirm): visible confirmation.
16. [89 button hold](REFACTOR-INPUT.md#change-89--89-button-hold): tap and hold.
17. [90 relative encoders](REFACTOR-INPUT.md#change-90--90-relative-encoders): endless knob values.
18. [91 SMC-PAD profile](SMC-PAD-INTEGRATION.md#change-91--91-smc-pad-profile): Linux measurement and knob strip.
19. [92 command panel](SMC-PAD-INTEGRATION.md#change-92--92-command-panel): controller flows and control pages.
20. [93 controller reach test](SMC-PAD-INTEGRATION.md#change-93--93-controller-reach): cover both stories.
21. [93a comfort](SMC-PAD-INTEGRATION.md#change-93a--93a-controller-comfort): panic, redo, date names, velocity curve.

Changes 88–90 can start without the device. Changes 91–93a need it. If the
device is not at hand, do 88–90 and continue with phase E. Return to the
remaining phase D items when the device arrives. The owner checks a full
session from an empty project to a saved one without the keyboard, except
for typed names.

### Phase E — effects and mix by ear

22. [pad-bus](FX-FILTERS.md#47-pad-bus): drive on the pad sum before inserts.
23. [drive-aa](FX-FILTERS.md#48-drive-aa): reduce aliasing in drive.
24. [filter-v2](FX-FILTERS.md#49-filter-v2): 24 dB slope, notch, peak, envelope, key tracking.
25. [reverb-v2](FX-FILTERS.md#410-reverb-v2): FDN, pre-delay, low cut, width.
26. [delay-v2](FX-FILTERS.md#411-delay-v2): filters in loop, ping-pong, tempo sync.
27. [fx-core](ROADMAP-DETAILS.md#fx-core): up to four inserts per pad and master.
28. [screen-layouts](UI-UX.md#43-screen-layouts): FX slots and sends. Files data and a preview wave.
29. [mixer-view](UI-UX.md#44-mixer-view): F8 mix screen.

The owner hears drum effects and sees a readable mix.

### Phase F — the glitch kit (0.4b)

30. [pad-pitch-env](ROADMAP-DETAILS.md#pad-pitch-envelope): pad pitch envelope.
31. [pad-loop](ROADMAP-DETAILS.md#pad-loop): loop with crossfade.
32. [step-locks](ROADMAP-DETAILS.md#step-locks): slice, start, reverse, pitch, cutoff.
33. [step-ratchet](ROADMAP-DETAILS.md#step-ratchet): ramps and probability.
34. [stutter](ROADMAP-DETAILS.md#stutter): held master effect.
35. [roll-matrix](ROADMAP-DETAILS.md#roll-matrix): rolls over the running pattern.
36. [stretch-offline](ROADMAP-DETAILS.md#stretch-offline): make a stretched sample offline.

The owner hears chopped, retriggered, and stretched breaks in live play.

### Phase G — tempo, resampling, and export

37. [102 sample tempo](SAMPLE-WORKFLOWS.md#change-102--102-sample-tempo): BPM, beats per sample, tempo-grid chop, repitch fit.
38. [record-latency](ROADMAP-DETAILS.md#record-latency): align recording to beat.
39. [105 resample in tempo](SAMPLE-WORKFLOWS.md#change-105--105-resample-in-tempo): record at project tempo.
40. [sample-ops](ROADMAP-DETAILS.md#sample-operations): trim, fades, reverse, DC removal.
41. [export](ROADMAP-DETAILS.md#export): render slots to WAV offline.
42. [project-safety](ROADMAP-DETAILS.md#project-safety): collect, relink, autosave, recover.

The owner checks a full track from an empty project to an exported WAV.

### Phase H — release 1.0

43. [format-freeze](ROADMAP-DETAILS.md#format-freeze): freeze project format.
44. [soak](ROADMAP-DETAILS.md#soak): four-hour session.
45. [packaging](ROADMAP-DETAILS.md#packaging): one binary and ptsr doctor.
46. [manual](ROADMAP-DETAILS.md#manual): installation and use.
47. [three release tracks and friction log](ROADMAP-DETAILS.md#release-track-review): V1 criterion 1.
48. [tag v1.0.0](ROADMAP-DETAILS.md#tag-v100): tag the accepted release.

The owner checks the package, manual, tracks, and friction log before the tag.

### Later

Move an item into a phase only after an owner decision in
[the decisions table](#decisions-to-get-from-the-owner).

- [compact-help](UI-UX.md#33-compact-help), [key-consistency](UI-UX.md#34-key-consistency),
  [style-doc-split](UI-UX.md#45-style-doc-split-between-two-changes), and the debug fixtures
  of [control-pages](ROADMAP-DETAILS.md#control-pages) beyond what 92 needs.
- [fx-cost](FX-FILTERS.md#43-fx-cost), unless the callback budget needs it earlier.
- Fine level matching, inter-sample peak precision, and
  [catalogue rules](CROSS-CUTTING.md#12-screen-catalogue-task-in-change-87).
- [App library split and merged branches](ROADMAP-DETAILS.md#later-internal-work).
- [Pad lights](SMC-PAD-INTEGRATION.md#change-94--94-pad-lights-optional-experiment) and terminal fallback.
- 1.x in [V1.md](V1.md#4-everything-else-is-1x): phrase variation, groove templates,
  break to pattern, chromatic and bass mode, sidechain, LFO and modulation.
  Other 1.x work includes MIDI clock and Link, cue output, loudness meter, layers,
  kits and presets, launcher and scenes, library, and time stretch.

## Review notes before implementation

These points qualify the detailed plans. Resolve each in that change's design and spec
before code. The imported plans do not override an existing requirement.

- **Count presses consistently (93):** A cell in a visible panel is one press.
  First entry is PAD BANK plus two layer 2 hits. The first hit only shows the panel.
  Count first entry and repeated commands separately. Test both from each screen and mode.
  Do not assume an unchanging panel row proves that a command is reachable inside a picker or a question.
- **Protect the latched panel too (92):** PAD BANK layer 2 needs no held button.
  A destructive cell asks a question before it removes or replaces anything.
  The rule applies to every panel access path. Keep X-Session direct defaults unchanged.
- **No hidden bug fixes (85):** the pending-action acknowledgement changes a race outcome.
  List it as an explicit fix with a scenario, not as a behavior-neutral code move. Resolve
  priority differences found by 84 before the refactor.
- **Preserve hit velocity (84):** the current UI `PadHit` event has no velocity. Carry the
  original velocity when Chop is routed to the UI. Do not replace it with a constant.
- **Keep hidden hits hidden (96):** filter raw hits outside the shortened length before
  quantization. Wrap a quantized hit at the loop boundary only after that filter. Otherwise
  shortening the pattern would move hidden hits into its beginning.
- **Controller-only work needs no fake keyboard equivalent:** test layer 2 notes,
  the first-hit rule, optional holds, and layer marks with timed MIDI events.
  Paired tests cover shared logical actions.
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
   Those exceptions are layer selection, its mark, and an optional quick look. The reach test of 93
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
| Phase D | Is the SMC-PAD at hand for changes 91–93a? | Do 88–90, then phase E. Return to 91–93a when the device arrives. |
| 88 | When does confirmation work resume? | In phase D. Test the keyboard path before the device check. |
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
| 92 | Is PAD BANK layer 2 the main panel path? | Yes. It latches the panel. |
| 92 | Does the first layer 2 hit only show a hidden panel? | Yes. It runs no cell. |
| 92 | Does holding STOP open the panel? | Off by default. An optional setting enables it. |
| 92 | How does Panic work? | A panel cell with a question-free double press. |
| 92 | May an SMC panel cell apply a layout or answer a replacement question? | Yes, with a visible question for destructive changes through every panel path. Keep X-Session direct defaults unchanged |
| 93a | Where are Panic, Undo, and Redo? | Fixed panel cells. Panic needs a double press. |
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
