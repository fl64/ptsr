## ADDED Requirements

### Requirement: Persistent save status

Every screen SHALL show `Unsaved` while changes await saving and `Save failed` after a write refusal.
`Save failed` SHALL take precedence until a save succeeds.
The indicator SHALL persist after the error notice ends and while another notice appears.
A successful save SHALL clear the indicator until another change occurs.

#### Scenario: Pending work has a visible save state
- **WHEN** a keyboard or controller action changes the session before autosave
- **THEN** the status shows `Unsaved` on Pads, Files, Chop, FX and Settings

#### Scenario: A save failure stays visible after its notice
- **WHEN** a save fails and its notice expires or another notice appears
- **THEN** `Save failed` remains visible in the shared status line on every screen, including small and large supported terminals
- **AND** no new mode opens, no row moves, and existing help, notices and controls remain available

#### Scenario: Successful retry clears the save warning
- **WHEN** a manual retry saves all pending work
- **THEN** neither `Unsaved` nor `Save failed` remains in the status
- **AND** a later edit shows `Unsaved` until its save succeeds
