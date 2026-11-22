## ADDED Requirements

### Requirement: Both inputs reach the pad sound shape

The keyboard SHALL reach SOUND through the current page cycle and use the normal parameter controls. SMC-PAD SHALL show a SOUND page action on P14 of the pad edit panel. SOUND SHALL show the row encoders, the current values and the way back after release of PAD BANK.

#### Scenario: Keyboard and raw controller events change the same sound shape

- **WHEN** the owner opens the pad sound page and changes its values by either input
- **THEN** both routes change the same pad settings
- **AND** the screen shows the settings and how to return to the pad grid

### Requirement: Both inputs have a one-cent tuning step

A coarse Fine tune step SHALL change five cents. The keyboard SHALL use Shift for one cent. SMC-PAD SHALL use E5 with a verified PAD BANK hold for one cent. E5 SHALL use one-cent steps when PAD BANK hold is off or unavailable. A learned fine control SHALL select one-cent steps for its device.

#### Scenario: The controller can tune one cent with or without a hold shortcut

- **WHEN** the owner turns E5 on SOUND during a verified PAD BANK hold or with the hold shortcut off
- **THEN** each detent changes Fine tune by one cent
- **AND** the hint shows the available route and the value remains visible after release

### Requirement: MIDI learn can assign the sound shape controls

Pan, Fine tune, Velocity, Curve and Hold SHALL be available for MIDI learn.

#### Scenario: A learned controller changes pad sound settings

- **WHEN** the owner learns a compatible control for a new sound shape parameter and then moves it
- **THEN** the assigned parameter changes for the selected pad
