## ADDED Requirements

### Requirement: Name a pad

The system SHALL open a name input for the selected pad on the pads screen with `Ctrl+N`.
The input starts with the displayed name. `Enter` saves the name, `Esc` cancels, and an
empty name restores the name from the sample file. The input SHALL accept text in both
keyboard layouts and SHALL NOT play pads while it is open. The system SHALL show the
custom name in the pad grid and save it in the project or scratch session. Old sessions
without a name SHALL open with the sample file name. Renaming SHALL NOT change the sound.
A successfully replaced or cleared sound SHALL clear the custom name. A failed load
SHALL keep the previous name. When pads are swapped, the name SHALL move with the sound.
A pad settings reset SHALL keep its name. Undo SHALL restore the name with the pad state.

#### Scenario: Rename a pad
- **WHEN** the owner selects a pad, presses `Ctrl+N`, types a name and presses `Enter`
- **THEN** the grid shows the name and the sound stays unchanged

#### Scenario: Cancel the pad name
- **WHEN** the owner edits a pad name and presses `Esc`
- **THEN** the name stays as it was

#### Scenario: Restore the file name
- **WHEN** the owner clears the pad name and presses `Enter`
- **THEN** the pad shows the file name again

#### Scenario: Pad name after restart
- **WHEN** the owner names a pad, saves the project and restarts
- **THEN** the same name shows on that pad

#### Scenario: Replace a named pad sound
- **WHEN** a new sample replaces the sound of a named pad
- **THEN** the pad shows the new sample file name

#### Scenario: A failed replacement keeps the name
- **WHEN** replacement of a named pad sound fails
- **THEN** the old sound and name stay

#### Scenario: Swap named pads
- **WHEN** two pads are swapped
- **THEN** each custom name moves with its sound

#### Scenario: Clear and undo a named pad
- **WHEN** the owner clears a named pad and undoes the clear
- **THEN** the sound and its custom name return
