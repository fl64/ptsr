# screen-help Delta

## MODIFIED Requirements

### Requirement: Truthful question controls

An existing question SHALL show ASK, its current target and effective keyboard answer controls. A controller answer SHALL appear only for an actual connected binding that reaches that answer through current production dispatch. A saved `ui.confirm` binding SHALL be eligible when it can answer this question. An unavailable controller path SHALL be omitted. Question safety, existing answers and non-answer input behavior SHALL stay unchanged.

#### Scenario: Questions show effective working answers
- **WHEN** an existing question opens with default keys, owner key overrides or a working connected controller binding
- **THEN** its answer line names only controls that actually confirm or cancel that question
- **AND** recorded keyboard/raw-MIDI runs through advertised paths retain the existing result and refusal behavior

#### Scenario: Missing controller confirmation stays honest
- **WHEN** a controller has no binding that answers the current question
- **THEN** the question shows its keyboard answers without claiming STOP, Apply or another unavailable controller path
- **AND** no new action or binding is created by drawing the answer line

#### Scenario: Learned shared confirmation appears only when it works
- **WHEN** the owner saves `ui.confirm` on a button and opens each current question with that controller connected
- **THEN** the question names the button and its press answers through the current target checks
- **AND** disconnecting the controller removes its answer label without changing keyboard answers
