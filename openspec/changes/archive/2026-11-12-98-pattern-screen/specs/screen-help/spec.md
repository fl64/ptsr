# screen-help Delta

## ADDED Requirements

### Requirement: Pattern view help names its keys and marks

When A1 exists, ordinary Pads help SHALL show the effective view keys. F1 SHALL explain the onset, between-line and multiple-hit marks, the playback plate, the all-pads row and the underlined banks. Owner overrides and `none` SHALL stay effective in help.

#### Scenario: Pattern view help names view keys and marks
- **WHEN** Pads shows A1 with default or owner-moved view keys, and the owner opens F1
- **THEN** the help line shows the effective view keys, and F1 explains each mark of the strip and the bank row
- **AND** a `none` binding removes the key from help without an invented fallback
