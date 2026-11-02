# keyboard Specification

## Purpose
TBD - created by archiving change 64-user-keymap. Update Purpose after archive.

## Requirements

### Requirement: Shared FX screen keys

The system SHALL open the shared FX screen with `F4`. On that screen `Tab` and
`Shift+Tab` SHALL select the next and previous shared effect block. `I` and `K` SHALL
select a parameter, and `J` and `L` SHALL change it. The pad keys SHALL play pads as on
other screens. The bottom help and F1 SHALL name the controls.

#### Scenario: Open the shared FX screen
- **WHEN** the owner presses F4
- **THEN** the shared FX screen opens and the selected block is clear

#### Scenario: Release a pad after the screen changes
- **WHEN** a MIDI control starts a gate pad, the owner opens F4 and releases that control
- **THEN** the pad stops even if that control has a different action on F4

#### Scenario: Play pads on the FX screen
- **WHEN** the shared FX screen is open and the owner hits a pad
- **THEN** the pad plays as it does on the pads screen

### Requirement: The owner's keymap
The system SHALL build the shipped keymap into the program and read the owner's keys from
`keymap.toml` in the user config folder (`~/.config/ptsr/`) over it, whatever
folder the program starts in. A key bound in a section of the user file replaces the
shipped binding of that key in that section; the action `none` takes a key away. A pad key
bound to a command, alone or with Shift, is refused as in the shipped keymap and named on
the status line; with `Ctrl` (`Ctrl+A`) it is a command key. The old
action name `pad.scope_next` is read as `pad.mark_bank`, in the keymap and in the learned
MIDI bindings. A file that cannot be read is named on the status line with its line, and
the program runs with the shipped keys.

#### Scenario: A key of my own
- **WHEN** the user keymap has `"Ctrl+G" = "pad.mark_bank"` under `[screens.pads]`
- **THEN** `Ctrl+G` marks the open bank on the pads screen, and `Ctrl+A` still does too

#### Scenario: An old action name
- **WHEN** the user keymap has `"Ctrl+G" = "pad.scope_next"` under `[screens.pads]`
- **THEN** `Ctrl+G` marks the open bank, and the status line names no error

#### Scenario: A key taken away
- **WHEN** the user keymap has `"Ctrl+A" = "none"` under `[screens.pads]`
- **THEN** `Ctrl+A` does nothing on the pads screen

#### Scenario: A broken file
- **WHEN** the user keymap has a line that cannot be read
- **THEN** the status line names the file and the line, and every shipped key works

#### Scenario: From any folder
- **WHEN** the program starts from the home folder and the user keymap binds `Ctrl+G`
- **THEN** `Ctrl+G` works, and the shipped keys work as when started from the project folder

#### Scenario: Pad keys stay pads
- **WHEN** the user keymap binds `Q` to a command
- **THEN** `Q` still plays its pad, and the status line says a pad key cannot be a command
