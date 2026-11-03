## Why

On Chop, a controller pad previews a slice by its pad number. A keyboard pad instead
follows the listen or assign mode. The owner cannot fill an assign plan from controller
pads as they can from the keyboard. Separate input tests do not prove that both paths
act the same. A source check also found that MIDI copy targeting can send a direct
pad hit before the replacement question. This breaks the silent copy rule from 83a.
Paired baseline tests then confirmed three more differences: MIDI swap loses target
selection and undo, file placement loses target selection, and a clear question does
not stop direct MIDI pad hits. The owner approved these narrow repairs in 84.

## What Changes

- Add one test driver for keyboard events and raw MIDI bytes through the real dispatch.
- Compare engine commands and visible state for existing pad workflows.
- Make controller pads on Chop use the same listen and assign logic as keyboard pads.
- Keep the original MIDI velocity when the UI plays a linked pad.
- Keep normal MIDI pad hits on their direct audio path and keep the X-Session defaults.
- Restore silent MIDI copy targeting from 83a, with the same selected target and
  replacement question as the keyboard. Keep releases of already held pads.
- Select the same target for MIDI file placement and swap; record the same swap undo step.
- Keep pad hits silent under an open question, without losing original held-pad releases
  or changing confirmation commands. Keep marking audible as before.
- Record other differences before deciding their scope. Do not hide extra fixes in this step.

## Capabilities

### Modified Capabilities

- `input-devices`: controller pad listening and assignment on Chop match keyboard pads;
  MIDI copy targeting follows the existing silent-copy contract; file and swap targets,
  swap undo and pad silence under questions match the existing keyboard route.

## Impact

Input and application tests, the MIDI pad-hit event, shared state publication, copy,
file and swap targets, question guards and the Chop input route. No new dependency,
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
3. Press P and hit target pads, including a pad in another bank. The same plan must fill.
   Apply it with the keyboard; controller pad hits must not bypass replacement questions.
4. Open a question or screen learn and hit pads. They must not play or change a plan.
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
7. Listen to `renders/84-input-parity.wav` after implementation. Paired hits must match;
   light and strong MIDI hits must differ in level, with no clicks or clipping.
