# Plan sections for the ordered roadmap

The [roadmap](README.md#ordered-roadmap) gives the work order. This file gives a section
to items that do not have one in another plan. These sections are guides, not OpenSpec
requirements. Each behavior change still needs its own proposal, design, spec, tests,
branch, and PR.

## Pitch quality

Change 110 adds anti-aliased interpolation for pitched pads and checks sample-rate
conversion on load. Set the alias limit and reference sounds in its design. See the
[sound-quality scope](V1.md#milestone-03--sound-quality-work).

## No clicks

Add short fades at starts, stops, steals, cuts, and slice edits. Keep source transients.
The owner checks each path by ear and with the automated click checks. See the
[no-clicks release criterion](V1.md#2-release-criteria).

## Control pages

`control-pages` is a documentation and drawing change. Make one control table for each
screen and mode, including Pattern. Each table names pad, knob, panel, and button roles.
Do not change an action in this change. Include a page for the chop and pad-edit flows
in [the SMC-PAD plan](SMC-PAD-INTEGRATION.md#2-the-layout). A unit test checks that
bank 1 knob 1 selects the current thing or moves its cursor on each page.

## FX core

Change 113 adds up to four serial inserts per pad and on the master. It includes the
crusher and lo-fi insert, a DJ filter, and a master clipper that defaults to off.
The [effect architecture](V1.md#milestone-05--tempo-resample-effects-and-export) gives
the chain, level, and render rules.

## Pad pitch envelope

Add a per-pad pitch envelope before step locks use pitch as a step value. Give the owner
an audible sweep and a pad edit control. The [1.0 sound scope](V1.md#milestone-05--tempo-resample-effects-and-export)
keeps it separate from sample tempo.

## Pad loop

Add loop playback for gate and one-shot pads with a crossfade at the seam. Test forward
and reverse directions. The [1.0 sound scope](V1.md#milestone-05--tempo-resample-effects-and-export)
sets this feature before the glitch kit.

## Step locks

Store slice, start, reverse, pitch, and cutoff on a hit. Keep the rest of change 104 in
1.x. See the [glitch kit scope](V1.md#milestone-04b--glitch-kit).

## Step ratchet

Repeat a hit inside its step. Include ramps and probability, with separate rules for
written hits and held performance. See the [glitch kit scope](V1.md#milestone-04b--glitch-kit).

## Stutter

Repeat a short part of the master sound in tempo while held. Keep the pattern clock
running beneath it. See the [glitch kit scope](V1.md#milestone-04b--glitch-kit).

## Roll matrix

Give the owner a 4×4 held roll layer over the running pattern. Check all 16 cells at
172 BPM. See the [glitch kit matrix](V1.md#milestone-04b--glitch-kit).

## Stretch offline

Stretch a pad or slice into a new sample outside the audio thread. Show progress and
put the result on a pad. See the [glitch kit scope](V1.md#milestone-04b--glitch-kit).

## Record latency

Change 112 measures input and output latency and aligns a recording to the beat.
Resampling in tempo depends on it. See the [sound-quality scope](V1.md#milestone-03--sound-quality-work).

## Sample operations

Add trim silence, fades, reverse, and DC removal as offline operations on samples.
Keep the original sample safe until a result is ready. See the
[sample workflow scope](V1.md#milestone-05--tempo-resample-effects-and-export).

## Export

Export pattern slots to WAV offline. The owner checks the output against playback and
completes a track from an empty project. See the
[release loop](V1.md#1-the-core-of-10).

## Project safety

Collect project samples, relink missing files, keep autosave versions, and recover
after a failed save. The [release safety scope](V1.md#milestone-06--data-safety-and-release)
and [sample workflow plan](SAMPLE-WORKFLOWS.md#self-contained-projects-owner-decision-2026-11-12-after-98c)
give the boundaries.

## Format freeze

Freeze the 1.0 project format. Keep fixtures for each saved version and test later
migrations against them. See the [format promise](V1.md#2-release-criteria).

## Soak

Run a four-hour session with a pattern, effects, and varied input. Define memory and
xrun limits before the test. See the [soak criterion](V1.md#2-release-criteria).

## Packaging

Ship one Linux binary and a `ptsr doctor` command. Check the package on a clean
machine. See the [install criterion](V1.md#2-release-criteria).

## Manual

Write a short manual for installation, the keyboard, the SMC-PAD, the sample-to-export
loop, and recovery. Check every procedure with the release build.

## Release track review

The owner makes three tracks and reviews their [friction log](../V1-FRICTION.md).
Each track starts with an empty project and ends with an exported WAV. See the
[three-track criterion](V1.md#2-release-criteria).

## Tag v1.0.0

Tag `v1.0.0` only after the owner accepts the release criteria, the three tracks,
the soak, the package, and the manual. The tag ends phase H.

## Later internal work

Keep the app library split, merged-branch cleanup, and terminal fallback after 1.0.
Also defer detailed peak precision, fine level numbers, and extra catalogue rules.
Move an item forward if it blocks an earlier phase or the owner records a decision.
