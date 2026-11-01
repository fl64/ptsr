# ptsr (PT's SampleR): project context

## What it is

A terminal sampler-groovebox for Linux. References: Roland SP-404 MK2 (live work) and Akai MPC (the program and pad model). The sound reference is glitch ambient (Access To Arasaka): chopped breaks, digital noise, stretched textures.

## Who it is for and how

One owner, a musician. The owner works without a mouse: keyboard and MIDI controllers (now M-Audio X-Session Pro, later M-VAVE SMC-PAD), and the owner can use them at the same time. An AI agent writes the code. The owner does not read the code. The owner checks the specifications, the renders and the program by hand.

## Current goal (stage 1)

Easy and clear, both from the keyboard and from MIDI:

1. load a sample,
2. chop it,
3. assign the slices (or whole samples) to pads,
4. play the pads.

Everything else (effects, sequencer, launcher, recording, PaulStretch) comes later. The vision is in `docs/VISION.md`. It is not mandatory.

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
