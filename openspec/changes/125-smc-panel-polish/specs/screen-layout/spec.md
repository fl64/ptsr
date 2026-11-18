## ADDED Requirements

### Requirement: SMC-PAD Help explains the current knob operations

Help from the SMC-PAD panel SHALL open F1 Controller for that device.
The diagram SHALL label the active roles. The selected knob SHALL show its bank,
physical number and full operation. On Chop, the footer SHALL distinguish the
slice pair from the region pair when both pairs use the shipped roles.

#### Scenario: Chop help identifies the slice and region knobs
- **WHEN** the owner opens controller Help on Chop and selects the two bound pairs
- **THEN** the detail names each physical knob and which bound it changes
- **AND** both supported sizes show the difference between slice and region
