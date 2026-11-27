## MODIFIED Requirements

### Requirement: A1 correction keys use contextual actions

In correction, J/L and Left/Right SHALL select time cells. I/K and Up/Down SHALL select an event. Alt+Up and Alt+Down SHALL keep their shared view action. U/O SHALL keep bank control. Ctrl+Z and Alt+Z SHALL retain their permitted Undo roles. On ordinary Pads, pad parameter keys SHALL keep their roles.

#### Scenario: Correction keys change the target without changing a pad
- **WHEN** the owner uses step, event, view and bank keys in correction, then leaves it and uses the same keys
- **THEN** correction selects its target without changing pad parameters or raw hits
- **AND** ordinary Pads keeps its pad actions and the overview uses its navigation

#### Scenario: Correction keys respect owner bindings and input owners
- **WHEN** the owner moves or disables a key, opens F1 or text, or holds an edit-layer key
- **THEN** effective bindings and input priority apply before correction actions
- **AND** pad-block keys remain pads and Shift on a pad never becomes a command

## ADDED Requirements

### Requirement: A1 overview browses time and bars

In the A1 overview, effective J/L and Left/Right SHALL move one time cell. Effective I/K and Up/Down SHALL move one bar. SMC-PAD E1 SHALL move cells and LEFT/RIGHT SHALL move bars when the panel is hidden. Help SHALL name the active controls. Pad play, transport, grid view, bank changes and A1 actions SHALL keep their roles.

#### Scenario: Overview browses cells and bars with both inputs

- **WHEN** the owner changes cell and bar with the keyboard or SMC-PAD while the A1 overview is open
- **THEN** the browse cursor moves through the same cells and bars
- **AND** the help shows the controls for the active mode

### Requirement: A1 overview keeps pad edits on Pads

While the A1 overview or Step Edit is open, commands to change, mark, clear, take or copy pads SHALL leave the pad data intact. A refused command SHALL name Pads as the place to edit. The help SHALL describe the active A1 actions. Undo and Redo SHALL keep their roles.

#### Scenario: Overview does not change the selected pad

- **WHEN** the owner tries to change, clear, copy or mark a loaded pad while A1 is open
- **THEN** its sound and settings remain and the help shows how to return to Pads
