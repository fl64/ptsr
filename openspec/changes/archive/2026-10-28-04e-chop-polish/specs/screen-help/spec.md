## ADDED Requirements

### Requirement: Help line fits the terminal
The system SHALL show the whole help line: when it does not fit on one row it continues on a second row, and the key of the main step of the screen comes first.

#### Scenario: Narrow terminal
- **WHEN** the terminal is 140 columns wide and the chop screen is open
- **THEN** every group of the help line is visible, `[Enter] to pads` among the first items
