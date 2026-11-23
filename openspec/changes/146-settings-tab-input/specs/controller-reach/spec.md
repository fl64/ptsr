## MODIFIED Requirements

### Requirement: Settings values have a controller route

On Audio and MIDI, SMC-PAD encoder 2 SHALL change the selected value with the
same bounds and result as the keyboard value keys. While a picker is open, its
controls SHALL preserve all Settings values.

#### Scenario: Settings knob changes only the selected value
- **WHEN** separate keyboard and raw SMC-PAD runs select the same Settings row and change its value once
- **THEN** both runs reach the same value and bounds
- **AND** a knob turn during a Paths folder picker changes only its selection

## ADDED Requirements

### Requirement: Settings row and value controls follow the tab

Input SHALL ignore the row controls and value controls of Settings.
Paths SHALL ignore the value controls until the owner opens its folder picker.
When the owner opens Paths, the selection SHALL stay on its row if it is valid.
If the row exceeds the last root row, the selection SHALL move to the last root row.
These rules SHALL apply to tab selection with Tab and with SMC-PAD LEFT and RIGHT.

#### Scenario: Input ignores row and value controls
- **WHEN** the owner selects Click level on Audio, opens Input and uses the row and value controls
- **THEN** the app keeps the earlier Audio row and its settings
- **AND** the config file stays the same

#### Scenario: Paths bounds its row and opens the selected root
- **WHEN** the owner opens Paths from a row past its last root and uses a value control
- **THEN** the app selects the last root row and preserves the audio settings and paths
- **AND** the choose action opens the picker of the visible root
- **AND** row controls can select the other root

### Requirement: Settings encoder hints show usable controls

The SMC-PAD strip and Controller Help SHALL show which encoders can act.
An unavailable encoder SHALL stay dim after a turn.
The Paths folder picker SHALL show the folder role for E1.

#### Scenario: Settings encoder hints follow the tab and picker
- **WHEN** the owner uses E1 or E2 on Audio, Input, MIDI, Paths or its folder picker
- **THEN** the strip shows which encoder can act in the current view
- **AND** a turn leaves an unavailable encoder dim
