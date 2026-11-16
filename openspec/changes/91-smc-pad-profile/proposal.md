# Proposal

## Why

The SMC-PAD is connected, but ptsr has only an invalid example profile. Its two USB input ports mirror pad and encoder messages. The active preset sends absolute encoder values, which stop at 0 and 127. The owner cannot use the device alone to select files or change values without end stops.

## What Changes

- Ship a profile for the measured USB preset. Map both physical pad layers to the screen from top to bottom. Keep pad velocity and original releases.
- Use only one copy of mirrored performance input. Give the five measured buttons safe starting actions. Keep existing owner assignments and X-Session roles.
- Read and check the active SMC-PAD preset before changing its encoder records to relative mode. Enable relative knob actions only after the device returns matching records. Leave other preset data unchanged.
- Show the current roles of the two knob banks. Provide a virtual MIDI replay path for the same profile and dispatch used in production.
- Report setup failure without false relative steps. Keep pads and buttons available when setup fails.

This change SHALL NOT add the command panel, pad lights, new sound processing or a new project-file format. Change 92 owns the command panel. Change 93 tests reach from every screen and mode.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `input-devices`: the measured profile, mirrored ports, guarded encoder setup and raw MIDI tests.
- `screen-layout`: visible SMC-PAD knob roles and physical pad order.

## Impact

The change affects device profiles, MIDI setup, input routing, the controller view and test tools. It adds no dependency or audio callback work.

## How to check

### Keyboard

1. Play the top-left and bottom-left screen pads with their keys. Check the same positions and the original release behavior.
2. Use the current keys to select a file, change a value and return from a screen. Check that all keys retain their roles.

### Controller

1. Connect the SMC-PAD by USB. Hit the top-left and bottom-left physical pads. Check that each sounds once at its screen position.
2. Use PAD BANK and hit those two positions. Check the second layer in F12 input debug. The panel does not exist yet, so these hits run no command.
3. Turn the first and last encoders in both knob banks in both directions. Check relative steps and the visible role names. Check that no value stops because the physical encoder reaches 0 or 127.
4. Press and release LEFT, RIGHT, PLAY, STOP and RECORD. Check the listed actions once per press. Unplug during a held gate and check its release.
5. Use the X-Session Pro at the same time. Check that neither controller doubles a hit or replaces existing assignments.

### Sound

This change adds no sound process or new WAV. Listen to an existing gate pad while changing its value, then disconnect the SMC-PAD. Check for a clean release.
