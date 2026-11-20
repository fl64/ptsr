## MODIFIED Requirements

### Requirement: An existing SMC-PAD map can adopt the reviewed layout

New SMC-PAD profiles SHALL use the shipped roles.
An existing map SHALL show old and proposed roles before it changes.
The review SHALL know the roles before and after change 124.
It SHALL keep owner bindings, removals and empty knobs unless the owner accepts a role.
An old role of unknown origin SHALL need approval.
The question before an update SHALL say «This cannot be undone.».
Only approved changes SHALL persist. Cancel or save failure SHALL keep the old map.
Other devices and USB preset records SHALL keep their state.

#### Scenario: Saved SMC-PAD roles survive the layout review
- **WHEN** the owner reviews a saved map with a personal, removed or ambiguous assignment
- **THEN** the review shows which roles can change
- **AND** the program keeps personal and removed assignments
- **AND** only the approved changes persist after restart

#### Scenario: Canceled or refused layout updates are safe to retry
- **WHEN** the owner cancels the review or the program cannot save the approved map
- **THEN** the old bindings still act and the other device keeps its own bindings
- **AND** a later successful retry applies the approved changes once

#### Scenario: Removed SMC-PAD knob stays unassigned on a saved map
- **WHEN** a saved map omits a knob role that the owner removed
- **THEN** a turn uses no factory fallback for that role
- **AND** the layout review does not restore the removed assignment

#### Scenario: A change 124 map removes the bank 2 pad roles
- **WHEN** the owner reviews a saved map with the change 124 bank 2 roles and applies all changes
- **THEN** bank 2 knobs 1–6 have no role after restart
- **AND** bank 2 knob 8 still changes master volume

#### Scenario: The layout update question says that it has no undo
- **WHEN** the owner asks for the SMC-PAD layout update
- **THEN** the question says «This cannot be undone.» before any role changes
