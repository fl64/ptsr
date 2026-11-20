## ADDED Requirements

### Requirement: Whole sample says what it does

The `whole sample` cell SHALL be dim while the region covers the whole sample. A press then SHALL show «the sample is already whole». A press on a trimmed region SHALL make the region the whole sample and SHALL show «region: whole sample».

#### Scenario: Whole sample on a trimmed region
- **WHEN** the owner trims the region with knobs 5 and 6 and presses `whole sample`
- **THEN** the region covers the whole sample and a notice says «region: whole sample»

#### Scenario: Whole sample on a whole region
- **WHEN** the region covers the whole sample and the owner presses `whole sample`
- **THEN** the cell is dim, the region stays and a notice says «the sample is already whole»
