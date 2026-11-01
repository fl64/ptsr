## ADDED Requirements

### Requirement: First steps hint
The system SHALL, while no pad holds a sample, say on the pads screen how to put one there.

#### Scenario: Empty session
- **WHEN** the program starts with no samples on pads and there is no other message
- **THEN** the hint line says "Pads are empty: F6 opens files, Enter takes one, a pad hit puts it there."

### Requirement: Grouped full help
The system SHALL show the full help (F1) grouped under headings by purpose, with keys of one kind on one row, fitting the screen.

#### Scenario: Banks in one row
- **WHEN** the owner opens F1 on the pads screen
- **THEN** banks 1–8 are one row "Ctrl+1…8", rows are under headings such as PADS, BANK, SCREENS

### Requirement: Help line per settings tab
The system SHALL list on the settings screen only the keys that work on the open tab.

#### Scenario: Audio tab
- **WHEN** the Audio tab is open
- **THEN** the help line has no "learn" and no "clear"
