## ADDED Requirements

### Requirement: P6 opens Step Edit on an empty A1

P6 edit steps SHALL use the F7 entry rules. An existing empty A1 SHALL
permit Step Edit when no other mode owns input. An absent A1 or an active
recording pass SHALL make P6 grey. The notice SHALL name the current refusal.
A press during count-in SHALL say «wait for the count-in».

#### Scenario: P6 and F7 can add the first A1 step
- **WHEN** the owner creates an empty A1 and opens Step Edit with P6 or F7
- **THEN** a pad hit adds one step, with no pad sound
- **AND** Undo removes that step
- **AND** the same entry request with no A1 leaves Step Edit closed
