## Why

The SMC-PAD file is an invalid template. The X-Session Pro file does not identify the
physical positions of its controls. Reliable profiles need a stated MIDI map and tests
that send the same bytes as the controllers.

## What Changes

- Replace the SMC-PAD template with a complete normal Note/absolute-CC profile.
- State the required SMC-PAD preset map. Do not claim it matches every factory preset.
- Keep the sixteen physical pad positions consistent with the screen grid.
- Give the first three knobs the same page roles in pad edit and on F4.
- Add explicit FX block controls and logical uses for the X-Session faders.
- Preserve existing user bindings. New defaults apply on first connection or after a
  confirmed reset to the profile.
- Test raw MIDI sequences through the real profile resolver, dispatch and application.

## Capabilities

### Modified Capabilities

- `input-devices`: complete documented profiles, physical controls and recorded MIDI tests.

## Impact

Device TOML files, input and application tests, user stories and live checks. No new
dependency or configuration format. No audio algorithm change.

Relative encoder protocols, device programming, BLE transport, LED feedback and changes
to the owner's current learned bindings are out of scope.

## How to verify

Run the recorded controller tests without hardware. Connect each controller, check its
messages on F12, and compare them with the documented map before resetting its bindings.
Play pads while the other controller changes pad and FX parameters. Check release,
velocity, pickup, undo and project persistence. Listen for clicks in the existing FX render.
