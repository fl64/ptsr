## MODIFIED Requirements

### Requirement: Binding changes are keyboard-only exceptions

Binding changes SHALL be F10 Learn, learn and forget in F12 MIDI, reset to profile and the SMC-PAD layout update. They SHALL be keyboard only. The controller SHALL select the control to bind or cancel with P1.

#### Scenario: Binding changes are keyboard only
- **WHEN** the reach inventory examines F10 Learn, learn and forget in F12 MIDI, reset to profile and the SMC-PAD layout update
- **THEN** it marks them as keyboard-only binding changes
- **AND** no panel view shows learn, learn control or forget control
