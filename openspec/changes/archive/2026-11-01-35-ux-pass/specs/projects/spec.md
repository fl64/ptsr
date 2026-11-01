## ADDED Requirements

### Requirement: The open project is marked
The system SHALL mark in the project picker the project that is open now, and SHALL show under the list the keys of the picker, not the keys of the screen beneath it.

#### Scenario: Which one is open
- **WHEN** "beat1" is open and the owner presses `Ctrl+P`
- **THEN** the row of "beat1" says it is open, and the help line names the picker keys: select, open, type a name to create, close
