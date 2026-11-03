# ptsr (PT's SampleR): project context

## What it is

A terminal sampler-groovebox for Linux. References: Roland SP-404 MK2 (live work) and Akai MPC (the program and pad model). The sound reference is glitch ambient (Access To Arasaka): chopped breaks, digital noise, stretched textures.

## Who it is for and how

One owner, a musician. The owner works without a mouse: keyboard and MIDI controllers (now M-Audio X-Session Pro, later M-VAVE SMC-PAD), and the owner can use them at the same time. An AI agent writes the code. The owner does not read the code. The owner checks the specifications, the renders and the program by hand.

## Current state and next goal

The sampler loads samples, chops them, assigns them to pads and plays them. Pad shaping,
microphone and output recording, projects, MIDI learn and shared FX are already present.
Change 82 (the F4 FX screen and safe physical X-Session defaults) was merged in PR #3,
accepted by the owner and archived on 2026-11-03. The SMC-PAD profile is still an incomplete
template. Transport and pattern sequencing are not implemented.

The next goal: every action that does not need typed text can be done from the SMC-PAD.
The keyboard stays the reference and keeps its controls. The work order is:

0. Keymap cleanup (83a). Ask the owner about J/L on Pads before code: proposed A changes
   values and moves banks to U/O. Reserve the later recording, redo and panic chords.
1. Input parity and refactors, then visible modes and new input controls (84–90).
2. A measured SMC-PAD profile, the command panel, reach checks and comfort (91–93a).
   Pad lights (94) are optional.
3. Transport, pattern play and recording, the step screen, slots, repeat and pad mute
   (95–100a). Input timestamps (101) are conditional on a measured or heard need.

Read [plans/README.md](plans/README.md) before preparing a change. Start with
[KEYMAP-CLEANUP.md](plans/KEYMAP-CLEANUP.md). The later parity draft is in
`openspec/deferred/84-input-parity/`; it waits for 83a. Each implementation gets its own
branch and active OpenSpec change. The preparation branch does not implement these features.

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
