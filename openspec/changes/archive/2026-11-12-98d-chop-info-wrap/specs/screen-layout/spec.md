# screen-layout Delta

## ADDED Requirements

### Requirement: The Chop slice line keeps each value whole

The Chop slice line SHALL wrap only between its ` · ` parts. A number SHALL stay on one row with its unit. A part wider than the column SHALL wrap by words. With the slice edit open, the first row SHALL keep the `EDIT · BOUNDS` badge.

#### Scenario: The Chop slice line keeps each value whole
- **WHEN** Chop shows the slice line at 80x26 and 120x40, with the slice edit closed and open
- **THEN** each of `start`, `length` and `cursor` stays on one row with its number and unit
- **AND** no text of the line goes away
