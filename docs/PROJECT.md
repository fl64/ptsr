# ptsr (PT's SampleR): project context

## What it is

A terminal sampler-groovebox for Linux. References: Roland SP-404 MK2 (live work) and Akai MPC (the program and pad model). The sound reference is glitch ambient (Access To Arasaka): chopped breaks, digital noise, stretched textures.

## Who it is for and how

One owner, a musician. The owner uses a keyboard and MIDI controllers without a mouse.
The current controller is M-Audio X-Session Pro. M-VAVE SMC-PAD is planned.
The owner can use the keyboard and controllers at the same time. An AI agent writes the code. The owner does not read the code. The owner checks the specifications, the renders and the program by hand.

## Current state and next goal

The sampler loads samples, chops them, assigns them to pads and plays them. Pad shaping,
microphone and output recording, projects, MIDI learn and shared FX are already present.
Change 82 (the F4 FX screen and safe physical X-Session defaults) was merged in PR #3,
accepted by the owner and archived on 2026-11-03. The SMC-PAD profile is still an incomplete
template. Transport and pattern sequencing are not implemented.

The next goal: every action that does not need typed text can be done from the SMC-PAD.
The keyboard stays the reference and keeps its controls. The work order is:

0. The owner accepted keymap cleanup (83a). Its archive is complete. J/L change values. U/O change banks.
   The owner accepted without completing the manual checks. The later recording,
   redo and panic chords stay reserved.
1. Input parity and refactors, then visible modes and new input controls (84–90).
2. A measured SMC-PAD profile, the command panel, reach checks and comfort (91–93a).
   Pad lights (94) are optional.
3. Transport, pattern play and recording, the step screen, slots, repeat and pad mute
   (95–100a). Input timestamps (101) are conditional on a measured or heard need.

Read [plans/README.md](plans/README.md) before preparing a change. Start with
[KEYMAP-CLEANUP.md](plans/KEYMAP-CLEANUP.md) for the shipped keys. The owner checked and
accepted input parity 84 on 2026-11-03. It is archived in
`openspec/changes/archive/2026-11-03-84-input-parity/`, based on 83a at `2b084d1`.
Its physical-controller pass remains unverified. Callback benchmark peaks above the
50% budget target remain unresolved. The owner accepted report-only change 84a on
2026-11-03. It is archived in `openspec/changes/archive/2026-11-03-84a-benchmark-report/`.
It adds wall time, thread CPU time, estimated gaps, exceedance counts and timer overhead
to the offline benchmark. Workload, sound, the 50% target and wall p99 exit checks stay
unchanged. Slow-core CPU spikes still need investigation.
The owner accepted input modes 85 on 2026-11-03. It is archived in
`openspec/changes/archive/2026-11-03-85-input-modes/`.
Shared pad handling, coherent MIDI state, ordered controller
commands and safe deferred releases are implemented. Separate repairs keep keyboard/MIDI
Chop holds independent and prevent late direct notifications from restarting Chop.
Input dispatch is in its own module. The move changed no method body. Offline checks
and the three 85 renders are ready. This is overall acceptance, not individual listening results.
Separate WAV listening remains unreported. Physical-controller behavior is still unverified.
No keys, profiles, saved-data format
or audio callback changed. Section 85 of `docs/LIVE-CHECKS.md` has the separate passes.
The owner requested `plan/controller-first-patterns` for 84, 84a and 85 as
exceptions to the separate-branch rule. Closure of 85 and work on 86 now share
`86-screen-registry` and one MR by owner request.
The owner accepted the screen registry (86) on 2026-11-04.
Its archive is `openspec/changes/archive/2026-11-04-86-screen-registry/`.
Implementation is complete at `a186fbb`. Commit `e90543c` records offline verification.
The registry keeps existing controls, help, learned assignments and FX rules for valid input.
Invalid shared MIDI state rejects new actions while preserving original releases and pending-input order.
Section 86 of `docs/LIVE-CHECKS.md` records verification and separate keyboard, controller and listening checks.
This is overall acceptance, not individual keyboard, physical-controller or WAV listening results.
The callback timing issue remains open. Report-only change 84b adds automatic local overrun records.
It does not change sound processing or exit gates. Callback correction remains separate work in 94a.
Raw CSV files, detailed machine logs and identifying local paths stay local.
The owner permits anonymized verification summaries in documents and commit history.
The owner accepted save retry (86b), safe project transitions (86c) and portable chops (86d) without the manual passes.
Their archives are `openspec/changes/archive/2026-11-05-86b-save-retry/`,
`openspec/changes/archive/2026-11-05-86c-safe-project-switch/` and
`openspec/changes/archive/2026-11-05-86d-portable-chops/`.
Keyboard, physical-controller and listening results remain unverified. Callback maxima above 50% remain unresolved.
The save-safety sequence is complete. The owner approved one PR for 84b and 86b–86d on `86b-save-retry`.
This is an explicit exception to separate change branches. Change 87 is not active and needs a separate request.
Activate only one change at a time. Push and PR need a separate request. The offline report checks are in `docs/LIVE-CHECKS.md`, section 84a.

[Vision version 3](VISION.md) keeps longer-term ideas, including the launcher, modulation,
parameter locks, slice motion and PaulStretch. It and `docs/plans/` are guides, not
requirements. Only the main specs and the active change authorize behavior.

## Principles

- Requirements are drafts. They change after each working version.
- Small changes. Check each one by ear and by hand.
- The sound never waits for the UI, the disk or MIDI.
- Logical actions are separate from input devices.
- UI texts are in English and short. Specifications are in English (see "Specification language"). Talk with the owner is in Russian.

## Stack

Rust stable, Linux, JACK/PipeWire, `midir`, `ratatui` + `crossterm` (kitty keyboard protocol), `symphonia`, `aubio`, `rtrb`, TOML configs. License GPL-3.0. The detailed rules are in `AGENTS.md`.

## Specification language

Requirements are written in English, in ASD-STE100 Simplified Technical English. Keep the OpenSpec keywords exactly in this form, because the validation needs them: "The system SHALL …", **WHEN** / **THEN** in scenarios.
