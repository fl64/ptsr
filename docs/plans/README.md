# Plans: controller-first work, SMC-PAD and the pattern sequencer

These plans were imported from the owner's eight planning documents on 2026-11-03.
The vision is in `../VISION.md`. The other seven documents are here.
These plans guide the agent, like `docs/VISION.md`. They are not requirements.
Requirements come only from `openspec/specs/` and the active change.
Each change starts with an OpenSpec proposal, design, spec deltas and tasks.
Follow `AGENTS.md`: one change at a time, specifications before code, a test for each scenario, atomic commits and a successful `task check`.
A change normally has its own branch. The owner requested continuation in `plan/controller-first-patterns` for this work.

## Current status (2026-11-04)

- Change 82 was merged in [PR #3](https://github.com/fl64/ptsr/pull/3). The owner accepted
  all remaining checks. It is archived in
  `openspec/changes/archive/2026-11-03-82-shared-fx-screen`.
- The owner requested continuation in `plan/controller-first-patterns`, including 84, 84a and 85.
  This is an exception to the separate-branch rule. The owner now requests closure of 85
  and work on 86 in one MR from `86-screen-registry`. Keep their commits separate.
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
  Earlier callback peaks and slow-core CPU spikes remain unresolved. Change 87 is next, but it is not active.
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

## Three tracks and the visual rules, in this order

| Order | Track | Document | Changes |
|---|---|---|---|
| 0 | Consistent keys and reserved recording / redo chords | `KEYMAP-CLEANUP.md` | 83a |
| 1 | Input parity, refactors and new input features. Change 87 is the visual base | `REFACTOR-INPUT.md`, `VISUAL.md` (87) | 84–90 |
| 2 | SMC-PAD integration and the command panel | `SMC-PAD-INTEGRATION.md` | 91–94 |
| 3 | Transport, patterns, recording, sequencer | `PATTERNS.md` | 95–101 |
| all | Tools, safety, comfort, docs and risks. Changes 93a and 100a | `CROSS-CUTTING.md` | tasks in many changes |

`VISUAL.md` also gives rules for every later change.
Each new mode or state brings its badge, pad glyphs, notices and hints for both inputs.

The order is not free. Track 2 needs the mode model, the button hold and the relative encoders
of track 1. Track 3 needs the screen registry for a new screen, plus `ui.confirm` and button holds (`erase`, note repeat).
It also needs the track 2 command panel for the new actions' controller paths.

```
83a keymap cleanup (A confirmed; specifications first)
    │
    ▼
84 parity ─► 85 modes ─► 86 screens ─► 87 badges ─► 88 confirm ─► 89 hold ─► 90 relative
                                                                                  │
            91 smc profile ◄──────────────────────────────────────────────────────┘
                 │
            92 command panel ─► 93 controller reach ─► 93a comfort ─► (94 pad lights, optional)
                 │
            95 transport ─► 96 pattern play ─► 97 pattern record ─► 98 step + screen
                                                                    │
                                              99 pattern slots ◄────┘
                                                   │
                                              100 note repeat ─► 100a pad mute ─► (101 input timestamps)
```

In parallel with 84, start step 0 of change 91 without code.
The owner measures the SMC-PAD on Linux and sets the encoders to relative. This removes the biggest risk early.

Change 82 is archived. The new prerequisite for 84 is keymap cleanup 83a.
Change 91 will replace change 83 (`openspec/deferred/83-controller-profiles`).
In 91, move its useful sources, pad orientation and raw-byte test approach into the new change.
Then remove the deferred folder. Complete 93 and 93a before transport work, so new actions have both input paths
and recording can use redo. Change 94 is optional and does not block 95.

Numbers are a proposal. If the owner inserts other work, renumber, but keep the order.

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
5. **Each sound change has a render** in `renders/` and the automatic checks (no NaN, no clip,
   timing ±1 sample).
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
| 83a | J/L on Pads: values with banks on U/O (A), or keep banks and require edit on FX too (B)? | The owner chose A on 2026-11-03. The archived 83a proposal records the choice |
| 87 | Yellow now means "the pads do something other than sound" (marks, in hand, assign, step, `erase`, panel). Change `docs/ui/STYLE.md`? | Yes |
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
| 99 | How many patterns play at the same time before a launcher exists? | One |
