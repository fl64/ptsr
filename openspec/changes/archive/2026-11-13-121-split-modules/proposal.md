## Why

The terminal drawing, application actions, and recorded input tests occupy three large files. This makes the next screen changes hard to review. Change 121 moves existing code into smaller modules before those changes.

## What Changes

- Move terminal drawing into modules for Pads, Chop, Files, FX, pattern views, Settings, and Help. Put shared badges, status, pad grid, and wave drawing in shared widget modules.
- Move screen-specific application methods from `main.rs` near the existing application modules.
- Move recorded input tests from `input_tests.rs` into topic modules. Keep their test names and bodies.
- Make one logical move per commit. Check the complete screen catalogue against the pre-change output after each drawing move.

This change includes no behavior change. It SHALL NOT change keys, input order, displayed cells, sound, saved data, configuration, or the audio callback.

The owner asked to continue on 2026-11-13 after the choice between this planned source split and pattern slots. This change follows the source split first, as the UI plan specifies.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. Existing specifications define the behavior. This change only moves its implementation and tests.

## Impact

The planned files are `crates/tui/src/lib.rs`, new `crates/tui/src/` modules, `crates/app/src/main.rs`, existing App modules, and `crates/app/src/input_tests/`. No dependency or file format changes.

## Out of scope

Do not rename actions, keys, labels, tests, or screen concepts. Do not repair a race or change a view during a move. If a behavior defect appears, record it for a separate change.

## How to check

### Keyboard

1. Open F2, F3, F4, F6, F12, and F1 in a disposable project. Check their current layout and help.
2. Hit and release a Gate pad on Pads and Chop. Check that only its original sound stops.
3. Change one pad value and one FX value with their current keys. Check that both values change once.

### Controller

1. Use the current learned screen actions to open Pads, Chop, FX, Files, and Settings.
2. Hit and release a Gate pad. Change one learned value. Check the same result as with the keyboard.

These checks need the owner's device. Recorded MIDI tests do not establish physical-controller acceptance.

### Views and sound

Compare every generated ANSI file in `renders/screens/` with the clean pre-change catalogue at 120x40 and 80x26. Inspect `pads-normal`, `chop-normal`, `files-normal`, `fx-normal`, `settings-normal`, `pads-pattern-overview-straight`, and `pads-help` at both sizes.

Listen to the existing `renders/82-shared-fx-screen.wav` and `renders/97-pattern-record.wav`. Sound must stay the same. This change adds no new WAV.
