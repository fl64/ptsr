## ADDED Requirements

### Requirement: A learned control hits the pad
The system SHALL make a control learned for "pad N" hit pad N of the current bank on every screen, with the velocity of the note or the value of the CC press; the release of the control (note off, CC 0) ends a gate pad.

#### Scenario: Hit on any screen
- **WHEN** a button is learned for "pad 1" and the owner presses it on the pads screen and on the files screen
- **THEN** pad 1 of the current bank plays both times

#### Scenario: Velocity
- **WHEN** the control is a note and the owner presses it with a low velocity
- **THEN** the pad plays quieter than a full-velocity press

#### Scenario: Gate by release
- **WHEN** pad 1 is in gate mode, plays from the learned control and the owner releases it (note off, CC 0)
- **THEN** the sound fades out right after the release, without a click

#### Scenario: Banks change under the button
- **WHEN** the owner switches from bank 1 to bank 3 and presses the control learned for "pad 2"
- **THEN** pad 2 of bank 3 plays
