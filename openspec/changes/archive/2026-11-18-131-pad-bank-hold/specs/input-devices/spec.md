## MODIFIED Requirements

### Requirement: SMC-PAD pads follow their physical rows

The supported SMC-PAD Master preset SHALL map channel 10 notes 36 through 51 to screen pads in physical rows from bottom to top. It SHALL keep each velocity and both note-off forms. Notes 52 through 67 SHALL use the same physical order. Without the PAD BANK hold shortcut, they SHALL be distinct second-bank command cells. With it, both ranges SHALL name the same physical pad. SHIFT SHALL have no invented host action.

#### Scenario: Physical pad rows match the screen
- **WHEN** raw Master MIDI presses and releases physical pads in the four corners of the first bank
- **THEN** each hit sounds once on the corresponding screen corner at its received velocity
- **AND** both note-off forms end the original Gate once

#### Scenario: Second pad bank stays distinct
- **WHEN** the hold shortcut is off and raw Master MIDI presses all 16 second-bank notes on channel 10
- **THEN** it recognizes 16 distinct cells ordered from the screen top row to bottom row
- **AND** no second-bank hit with a shipped panel binding plays a sample
