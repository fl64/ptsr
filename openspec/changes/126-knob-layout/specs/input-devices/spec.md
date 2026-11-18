## ADDED Requirements

### Requirement: Relative knobs use one step per detent

Each relative detent SHALL change its target by one step at any turn speed.
The system SHALL not multiply steps by turn speed.
A neutral CC SHALL not count as a detent.

#### Scenario: Fast turn uses single steps
- **WHEN** one knob sends four detents within 40 ms, then four more while the owner holds a global fine button
- **THEN** each detent changes the target by one step
- **AND** the fine button release changes no value

## MODIFIED Requirements

### Requirement: A held button selects fine steps

A global `knob.fine` button action SHALL set fine mode on its port while held.
Its original release, an invalid-state release or a disconnect SHALL clear fine mode.
Fine mode SHALL not change stored precision.

#### Scenario: Two devices keep separate step state
- **WHEN** two ports send the same CC and only the first port holds its fine button
- **THEN** the second port keeps its normal steps
- **AND** disconnecting the first port clears only its own fine state

### Requirement: An existing SMC-PAD map can adopt the reviewed layout

New SMC-PAD profiles SHALL use the shipped roles.
An existing map SHALL show the old and proposed roles before it changes.
The review SHALL accept the roles before change 124 and the roles of change 124.
The program SHALL keep owner bindings and removals.
An empty saved knob position SHALL remain unassigned unless the owner accepts a new role.
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

#### Scenario: Removed SMC-PAD knob stays unassigned on a saved map
- **WHEN** a saved map omits a knob role that the owner removed
- **THEN** a turn uses no factory fallback for that role
- **AND** the layout review does not restore the removed assignment

#### Scenario: A change 124 map removes the bank 2 pad roles
- **WHEN** the owner reviews a saved map with the change 124 bank 2 roles and applies all changes
- **THEN** bank 2 knobs 1–6 have no role after restart
- **AND** bank 2 knob 8 still changes master volume

## REMOVED Requirements

### Requirement: Relative speed and fine mode stay per device

**Reason**: The owner removed knob acceleration. A short turn moved a value through its full range.
**Migration**: `Relative knobs use one step per detent` gives the step rule.
