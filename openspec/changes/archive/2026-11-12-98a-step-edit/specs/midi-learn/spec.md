# Spec Delta

## ADDED Requirements

### Requirement: A1 correction actions can be learned

F12 MIDI SHALL list `pattern.step_edit`, `pattern.step_prev`, `pattern.step_next`, `pattern.event_prev` and `pattern.event_next` as button actions. Existing view, bank and Undo actions SHALL keep their learned routes. Learning SHALL keep confirmation, clearing, persistence and layer priority. No factory mapping SHALL be added.

#### Scenario: Learned correction buttons persist
- **WHEN** the owner learns note or CC buttons for correction, step and event actions, then saves and reloads
- **THEN** the actions remain available without replacing other assignments or factory defaults
- **AND** learning selection, cancellation and release do not execute an edit action

#### Scenario: Keyboard and raw MIDI correct the same hit
- **WHEN** keyboard and learned raw MIDI routes remove the same event from separate copies of A1
- **THEN** both runs keep identical raw rows, settings, Undo order and silent edit behavior
- **AND** later ordinary pad presses keep Shift velocity and actual MIDI velocity
