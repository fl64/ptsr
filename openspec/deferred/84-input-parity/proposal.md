## Why

On Chop, a controller pad previews a slice by its pad number. A keyboard pad instead
follows the listen or assign mode. The owner cannot fill an assign plan from controller
pads as they can from the keyboard. Separate input tests do not prove that both paths
act the same.

## What Changes

- Add one test driver for keyboard events and raw MIDI bytes through the real dispatch.
- Compare engine commands and visible state for existing pad workflows.
- Make controller pads on Chop use the same listen and assign logic as keyboard pads.
- Keep the original MIDI velocity when the UI plays a linked pad.
- Keep normal MIDI pad hits on their direct audio path and keep the X-Session defaults.
- Record other differences before deciding their scope. Do not hide extra fixes in this step.

## Capabilities

### Modified Capabilities

- `input-devices`: controller pad listening and assignment on Chop match keyboard pads.

## Impact

Input and application tests, the MIDI pad-hit event and the Chop input route. No new
dependency, key, profile default, audio algorithm or saved file format.

## Out of scope

The mode model and pending-action race fix (85), screen registry, visual redesign,
confirm, button holds, relative encoders, the SMC-PAD profile, command panel and patterns.
This draft is not active and has no implementation or completed test tasks.

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
5. Listen to `renders/84-input-parity.wav` after implementation. Paired hits must match;
   light and strong MIDI hits must differ in level, with no clicks or clipping.
