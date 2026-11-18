## ADDED Requirements

### Requirement: An existing SMC-PAD map can adopt the reviewed layout

New SMC-PAD profiles SHALL use the shipped roles.
An existing map SHALL show the old and proposed roles before it changes.
The program SHALL keep owner bindings and removals.
An old role with unknown origin SHALL require approval before replacement.
Only approved changes SHALL persist. Cancel or save failure SHALL keep the old map.
Other device profiles and USB preset records SHALL keep their state.

#### Scenario: Saved SMC-PAD roles survive the layout review
- **WHEN** the owner reviews a saved map with a personal, removed or ambiguous assignment
- **THEN** the review shows which roles can change
- **AND** the program keeps personal and removed assignments
- **AND** only the approved changes persist after restart

#### Scenario: Canceled or refused layout updates are safe to retry
- **WHEN** the owner cancels the review or the program cannot save the approved map
- **THEN** the old bindings still act and the other device keeps its own bindings
- **AND** a later successful retry applies the approved changes once
