## MODIFIED Requirements

### Requirement: Truthful question controls

An existing question SHALL show ASK, its current target and effective keyboard answers. The help SHALL name only connected controller inputs that can answer through normal dispatch. This includes a learned `ui.confirm` when it can answer. Binding questions SHALL show keyboard confirmation and any working cancel path. Other questions SHALL keep their safety rules and answers.

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
- **THEN** the question names the button only if its press can answer the current question within the current target limits
- **AND** disconnecting the controller removes its answer label without changing keyboard answers
