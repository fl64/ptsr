## Why

On Chop, a controller pad previews a slice by its pad number. A keyboard pad instead
follows the listen or assign mode. The owner cannot fill an assign plan from controller
pads as they can from the keyboard. Separate input tests do not prove that both paths
act the same. A source check also found that MIDI copy targeting can send a direct
pad hit before the replacement question. This breaks the silent copy rule from 83a.
Paired baseline tests then confirmed three more differences: MIDI swap loses target
selection and undo, file placement loses target selection, and a clear question does
not stop direct MIDI pad hits. The owner approved these narrow repairs in 84.
During the keyboard check, the owner also found that a linked slice sounds on a pad
hit in Chop but has no moving green playback line. Space already has that line.
The owner approved adding the line for pad playback, without changing sound or edit
commands. During implementation, the owner also requested a fixed-width LISTEN badge
that lights blue only while playback runs, keeping the play icon.

## What Changes

- Add one test driver for keyboard events and raw MIDI bytes through the real dispatch.
- Compare engine commands and visible state for existing pad workflows.
- Make controller pads on Chop use the same listen and assign logic as keyboard pads.
- Keep the original MIDI velocity when the UI plays a linked pad. Keep its original
  release target after bank/screen changes, without adding releases for silent hits.
- Pass a keyboard pad held before F10 through screen learn on release, preserving its
  original pad after bank/screen changes. Keep new learn hits silent and bindings unchanged.
- Keep normal MIDI pad hits on their direct audio path and keep the X-Session defaults.
- Restore silent MIDI copy targeting from 83a, with the same selected target and
  replacement question as the keyboard. Keep releases of already held pads.
- Select the same target for MIDI file placement and swap; record the same swap undo step.
- Keep pad hits silent under an open question, without losing original held-pad releases
  or changing confirmation commands. Keep marking audible as before.
- Show the moving green playback line for a pad hit in the open Chop editor too.
  Read the real sounding position without another audio hit. Keep Space, M, the
  owner-controlled edit cursor, pad sound and existing selection behavior unchanged.
- Keep the LISTEN badge at 10 cells including padding, dim at rest and blue with `▶`
  while the Chop preview or a linked pad of this chop plays. Do not shift the file name
  or change the assign badge.
- Record other differences before deciding their scope. Do not hide extra fixes in this step.

## Capabilities

### Modified Capabilities

- `input-devices`: controller pad listening and assignment on Chop match keyboard pads;
  MIDI copy targeting follows the existing silent-copy contract; file and swap targets,
  swap undo and pad silence under questions match the existing keyboard route;
  linked Chop gate pads keep their original release route; keyboard pads held before
  F10 release through learn without letting new hits sound.
- `chopping`: show the existing green playback line for a sounding pad slice too,
  with a fixed-width LISTEN badge that lights only during playback; keep transport
  and edit commands unchanged.

## Impact

Input and application tests, the MIDI pad-hit event, shared state publication, copy,
file and swap targets, question guards, the Chop input route and read-only pad playback
position reporting from audio to the waveform. No new dependency,
key, profile default, audio algorithm or saved file format.

## Out of scope

The mode model and pending-action race fix (85), screen registry, visual redesign,
confirm, button holds, relative encoders, the SMC-PAD profile, command panel and patterns.
This change is active on `plan/controller-first-patterns` by owner request after the
83a archive (`2b084d1`). The branch exception and baseline are recorded in `design.md`.
No push or PR is authorized.

## How to verify

Use a pad controller with default notes 36–51, or an explicit existing pad binding.
The X-Session Pro has no pads; keep its knobs and safe buttons as a second input.
Do two passes, one with keyboard pads and one with controller pads:

1. Open a cut sample on Chop before any slices are assigned. Hit pads in two banks.
   The same slices must play and become selected.
2. Assign slices to pads. Change a pad's pitch or filter. Hit it on Chop: both inputs
   must play that pad's sound and select its linked slice. A light MIDI hit stays quiet.
   Set it to gate, hold it, change bank, open FX with F4 and release the original key
   or controller pad. The original sound must stop; no new-bank pad must release.
3. Press P and hit target pads, including a pad in another bank. The same plan must fill.
   Apply it with the keyboard; controller pad hits must not bypass replacement questions.
4. Open a question or screen learn and hit pads. They must not play or change a plan.
   Also hold a gate pad before F10, change bank or screen, then open learn and release
   the original control. Only the original pad must stop; learn and its pending binding
   must stay open. A new pad key in learn must not stop a held MIDI pad.
5. On Pads, take a copy with Ctrl+C. Hit an empty target in another bank, then repeat
   with an occupied target. Both input passes must select the same target without sound.
   Only Enter may replace it; Esc keeps the copy in hand. Hold a gate pad before taking
   the copy, then release it: the original sound must stop.
6. Take a file in hand and hit an occupied target in another bank. Both input passes
   must select the target named in the question without sound. Cancel with Esc.
   Swap two loaded pads with Ctrl+X and a target hit: both inputs must select the target,
   and one Ctrl+Z must restore both pads. Mark two loaded pads and press Backspace:
   pad hits must stay silent under the question. Release a gate pad held before it;
   the original pad must stop even after the bank and screen change.
7. In listen and slice edit, hit a linked pad on Chop. The green line must follow
   its sounding position and disappear when it ends. Check pitch and gate release.
   The owner's cursor must not follow that line; Space and M must work as before.
   LISTEN must stay the same width, dim at rest and blue with `▶` while the preview
   or a matching pad sounds, also when its position is outside the waveform view.
   Repeat with raw notes or a learned pad button when hardware is available.
8. Listen to `renders/84-input-parity.wav` after implementation. Paired hits must match;
   light and strong MIDI hits must differ in level, with no clicks or clipping.

The owner has no controller available at this check. The physical-controller pass
remains unverified, not passed or waived. Raw-MIDI automated tests still apply.
