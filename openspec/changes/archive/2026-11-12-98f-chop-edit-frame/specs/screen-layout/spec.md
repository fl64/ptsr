# screen-layout Delta

## MODIFIED Requirements

### Requirement: The Chop slice line keeps each value whole

The Chop slice line SHALL wrap only between its ` · ` parts. A number SHALL stay on one row with its unit. A part wider than the column SHALL wrap by words. While the slice edit is open, the edit frame SHALL take the place of the slice line. "The slice edit uses the pad edit frame" sets that frame.

#### Scenario: The Chop slice line keeps each value whole
- **WHEN** Chop shows the slice line at 80x26 and 120x40 with the slice edit closed
- **THEN** each of `start`, `length` and `cursor` stays on one row with its number and unit
- **AND** no text of the line goes away
