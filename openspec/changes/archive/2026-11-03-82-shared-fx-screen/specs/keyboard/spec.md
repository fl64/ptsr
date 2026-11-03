## ADDED Requirements

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
