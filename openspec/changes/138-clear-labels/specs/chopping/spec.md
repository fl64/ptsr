## ADDED Requirements

### Requirement: Reset trim says what it does

The `reset trim` cell SHALL be dim while the region covers the reset trim. A press then SHALL show «the sample is already whole». A press on a trimmed region SHALL make the region the reset trim and SHALL show «region: reset trim».

#### Scenario: Whole sample on a trimmed region
- **WHEN** the owner trims the region with knobs 5 and 6 and presses `reset trim`
- **THEN** the region covers the reset trim and a notice says «region: reset trim»

#### Scenario: Whole sample on a whole region
- **WHEN** the region covers the reset trim and the owner presses `reset trim`
- **THEN** the cell is dim, the region stays and a notice says «the sample is already whole»

### Requirement: Remove all cuts

Chop SHALL have `remove all cuts` on device pad P6 and on the Delete key. It SHALL be dim while the chop has no cut. A press SHALL ask «Remove N cuts?» with P4 yes and P1 no. After yes, the chop SHALL have one slice. Pads linked to the old slices SHALL keep their sound, as after `equal parts`. A notice SHALL say «N cuts removed · P2 undo brings them back».

#### Scenario: Remove all cuts asks and can be undone
- **WHEN** the chop has 6 cuts and the owner presses `remove all cuts` and answers yes
- **THEN** the chop has one slice and a notice says «6 cuts removed · P2 undo brings them back»
- **AND** one undo brings back the 6 cuts

#### Scenario: Remove all cuts without cuts is dim
- **WHEN** the chop has no cut and the owner presses `remove all cuts`
- **THEN** the cell is dim and a notice says «no cuts to remove»
