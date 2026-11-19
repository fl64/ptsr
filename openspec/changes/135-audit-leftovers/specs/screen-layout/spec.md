## ADDED Requirements

### Requirement: Chop notices keep the wave fields readable

A notice on Chop SHALL use the notice row. It SHALL NOT cover the slice time, length, cursor or threshold fields.

#### Scenario: A hits notice leaves the fields readable
- **WHEN** the owner runs hits on Chop at 80×26 and at 120×40
- **THEN** the notice shows in the notice row and the threshold and time fields read in full

### Requirement: The mode line fits at 80 by 26

At 80×26, the mode line SHALL keep the apply and cancel controls visible. Hint columns SHALL give way to the mode line.

#### Scenario: Assign keeps apply and cancel at 80 by 26
- **WHEN** the owner opens Assign and hits a pad at 80×26
- **THEN** the mode line names the apply and the cancel control in full
