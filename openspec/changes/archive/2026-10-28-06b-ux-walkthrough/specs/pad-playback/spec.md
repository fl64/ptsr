## ADDED Requirements

### Requirement: Empty pads look empty
The system SHALL call a pad without a sample "empty" in the grid and show its test tone only in the pad info.

#### Scenario: New pad
- **WHEN** a pad has no sample
- **THEN** its cell says "empty" and, when it is selected, the info says "empty · test tone A2"
