## ADDED Requirements

### Requirement: Pattern keys keep their roles on the Pattern screen

The current pattern keys SHALL keep their roles on the Pattern screen.
These keys are record, erase, pass undo, setup, tap tempo, view grid, all rows, step edit, play and stop. Other screen keys SHALL keep their existing screen roles.

#### Scenario: Pattern keys act on the Pattern screen
- **WHEN** the owner uses Ctrl+R, Alt+E, Alt+Z, Alt+N, Alt+B, Alt+Up, Alt+Down, F7 and Space on the Pattern screen
- **THEN** each key does what it did in the old overview and the Pattern screen stays open
- **AND** F2, F3, F4, F6 and F12 open their own screens

## MODIFIED Requirements

### Requirement: A1 overview has a direct screen key

F5 SHALL open the Pattern screen from an ordinary screen. It SHALL take the place of the pad grid and the pad inspector.
F2 and unclaimed Esc SHALL return to Pads with the same bank, selected pad and view division.
The Pattern screen SHALL work with absent, empty, stopped and playing A1. Opening it SHALL NOT create a pattern or move transport.
Pad keys SHALL play and select pads as on Pads. Shift on a pad SHALL give full velocity.
Releases SHALL keep their original sound owner across screen and bank changes.

#### Scenario: Overview opens without editing or stopping A1
- **WHEN** the owner opens the Pattern screen with F5 while the pattern is absent, empty, stopped or playing
- **THEN** it shows the correct state without creating a pattern, restarting transport or changing recorded data
- **AND** F2 or Esc returns to Pads with the same bank, selected pad and view division

#### Scenario: Overview pad input keeps sound ownership
- **WHEN** a keyboard or raw MIDI pad starts a Gate on the Pattern screen and the owner changes bank or screen before release
- **THEN** the pad plays with its original velocity and its release stops the original note
- **AND** no new-bank sound or pattern edit follows the old release

### Requirement: Overview can show empty banks from the keyboard

Alt+H SHALL add or remove the empty rows of the selected pad's bank while the Pattern screen is open.
The key SHALL have no effect on pad mode or the pattern. Other screens SHALL keep their existing Alt+H behavior.
Owner overrides and `none` SHALL keep their existing priority rules.

#### Scenario: Overview empty-bank key keeps pad and pattern state
- **WHEN** the owner presses Alt+H on the Pattern screen, then presses it again
- **THEN** the empty rows of the selected bank appear and disappear without changing the current bank, pad modes or the pattern

### Requirement: A1 overview browses time and bars

On the Pattern screen, effective J/L and Left/Right SHALL move one time cell. Effective I/K and Up/Down SHALL move one bar. SMC-PAD E1 SHALL move cells and LEFT/RIGHT SHALL move bars when the panel is hidden. Help SHALL name the active controls. Pad play, transport, grid view, bank changes and pattern actions SHALL keep their roles.

#### Scenario: Overview browses cells and bars with both inputs

- **WHEN** the owner changes cell and bar with the keyboard or SMC-PAD while the Pattern screen is open
- **THEN** the browse cursor moves through the same cells and bars
- **AND** the help shows the controls for the active mode

### Requirement: A1 overview keeps pad edits on Pads

While the Pattern screen or Step Edit is open, commands to change, mark, clear, take or copy pads SHALL leave the pad data intact. A refused command SHALL name Pads as the place to edit. The help SHALL describe the active pattern actions. Undo and Redo SHALL keep their roles.

#### Scenario: Overview does not change the selected pad

- **WHEN** the owner tries to change, clear, copy or mark a loaded pad while the Pattern screen is open
- **THEN** its sound and settings remain and the help shows how to return to Pads
