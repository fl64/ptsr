## ADDED Requirements

### Requirement: SMC-PAD warns about shifted notes and a wrong preset

An unassigned out-of-range pad press on the Master port SHALL warn once per
connection. A verified wrong preset SHALL warn with its number and block knobs.
F1 and the setup guide SHALL explain the device Note Repeat clock and the
supported ptsr Note Repeat path.

#### Scenario: Shifted pads warn once on the Master connection
- **WHEN** a Master connection sends repeated unassigned pad notes outside the shipped ranges
- **THEN** the first hit warns and the notes cause no action
- **AND** other ports and channels do not give this warning

#### Scenario: A wrong preset gives its number and blocks knob turns
- **WHEN** the USB status reply gives an active preset other than the supported preset
- **THEN** ptsr reports the preset number and blocks knob turns
- **AND** the correct preset and encoder check let knob turns act again

#### Scenario: SMC-PAD help names the device repeat clock
- **WHEN** the owner reads the connected SMC-PAD map in F1
- **THEN** it explains the device clock and names ptsr Note Repeat
