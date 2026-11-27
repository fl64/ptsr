## MODIFIED Requirements

### Requirement: Modes show their state where they start

Each active mode SHALL have a mode line on the screen where it starts. The slice edit SHALL show `EDIT slice N · BOUNDS` and its close control. The copy line SHALL name the source sound, bank and pad. After the first SMC-PAD tap, the line SHALL say «TAP · 1 hit · tap again». With PAD BANK set to Off and no mode, the line SHALL say «PAD BANK: hit to show commands».

#### Scenario: The slice edit shows its close control
- **WHEN** the owner opens the slice edit with the SMC-PAD and the panel is hidden
- **THEN** the mode line says `EDIT slice 1 · BOUNDS` and ends with «PAD BANK + 1 close»

#### Scenario: The copy line keeps its source after a bank change
- **WHEN** the owner copies pad 1 of bank 1 and moves to bank 2
- **THEN** the mode line names the source sound and B1/P1

#### Scenario: Tap tempo confirms the first tap
- **WHEN** the owner taps tempo once on A1 from the SMC-PAD
- **THEN** the mode line says «TAP · 1 hit · tap again»
