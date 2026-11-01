## ADDED Requirements

### Requirement: Enter starts the order from the selected pad
The system SHALL, in the assign mode "all in order" with no start pad hit yet, take the
selected pad as the start on `Enter` and show the plan; the next `Enter` applies it as any
plan, with the overwrite question when pads are taken. A pad hit before `Enter` sets the
start as before.

#### Scenario: Enter with no pad hit
- **WHEN** pad Q of bank 2 is selected, the owner opens the assign mode, switches to "in
  order" with `Tab` and presses `Enter`
- **THEN** the plan lays slice 1 on pad Q of bank 2 and the rest after it, and the
  summary is shown; a second `Enter` applies it
