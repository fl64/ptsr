## ADDED Requirements

### Requirement: Save before project transitions

Opening another project or creating a new project SHALL save pending source work before replacing the current session.
Source work SHALL include scratch and named projects.
A save refusal SHALL keep pads, chops, undo, pending work, picker selection and the typed name.
New target folders and files SHALL NOT be created before a required source save succeeds.
Existing keys, startup read protection and sound isolation SHALL stay unchanged.

#### Scenario: Project edits survive an immediate switch
- **WHEN** the owner changes a project and selects another project before autosave
- **THEN** the source saves before the target replaces it
- **AND** returning to the source restores the saved pad and chop changes

#### Scenario: A save refusal keeps the selected target
- **WHEN** the owner selects another project while pending source work cannot save
- **THEN** each Enter reports refusal and retains the current pads, chops, undo and pending timestamp
- **AND** the picker stays open with the same selection and the target stays unchanged

#### Scenario: A save refusal keeps the new project name
- **WHEN** the owner enters a new project name while pending source work cannot save
- **THEN** each Enter retains the typed name and current work without creating that target
- **AND** after the write problem ends, Enter saves the source and creates the target

### Requirement: Selecting the current project does not reload

Selecting the current project's folder SHALL close the picker without reading, saving or reloading the session.
Pending work, undo, loader generations and current sound state SHALL stay unchanged.
Equivalent canonical folder paths SHALL count as the same folder when both paths exist.

#### Scenario: Selecting the current project keeps pending work
- **WHEN** the owner selects the open project, including a folder alias, while changes are pending
- **THEN** the picker closes without losing the changes or undo history
- **AND** no session save, loader reset or audio epoch change occurs

### Requirement: Validate a target before replacing the current project

An existing target SHALL pass session reading and validation before source saving or state replacement.
A damaged, newer, unreadable or missing target session SHALL keep the source project, save flags and last-project setting unchanged.
A target-creation refusal after source saving SHALL keep the source and picker open, with that source work safely saved.

#### Scenario: A target read refusal preserves the source
- **WHEN** the owner selects a target whose session is damaged, newer, unreadable or missing
- **THEN** the current session, pending work, undo, picker selection and last-project setting stay unchanged
- **AND** the refusal does not write either session or replace the source read-protection reason

#### Scenario: A target creation refusal retains saved source work
- **WHEN** source saving succeeds but the new target cannot be created
- **THEN** the source and its undo remain available and its pending flag reflects the successful save
- **AND** the picker keeps the typed name for a later retry
