## Why

The shared effects change every pad that sends to them, but their controls are inside the
pad edit. A separate screen makes their scope clear.

## What Changes

- Open the shared FX screen with `F4`. Select FX 1 and FX 2 with `Tab` and `Shift+Tab`.
- Use the same three controller knobs for the selected slot: type and two parameters.
- Keep drive and sends in the pad edit. Pads stay playable on the FX screen.
- Keep project settings and existing learned controls compatible.
- Replace the X-Session Pro defaults with physical navigation, preview, page and fader roles.
  Keep existing user assignments until a confirmed Reset to profile.

## Capabilities

### Modified Capabilities

- `editing`: shared effect pages leave the pad edit.
- `pad-playback`: shared effects are controlled on the FX screen.
- `keyboard`: F4 opens the FX screen and Tab selects its slot.
- `input-devices`: documented X-Session physical controls and safe default buttons.
- `midi-learn`: reset and clear scenarios use the replacement defaults.

## Impact

UI, keyboard and MIDI routing, help and tests. No audio format or dependency change.
Adding more effects or effect slots is out of scope.

## How to verify

Send two pads to FX 1. Open F4, select FX 1 and change its type and parameters. Both pads
must use the same effect. Press Tab and use the same knobs on FX 2. Restart and check the
settings. Listen for clicks while changing the controls.
