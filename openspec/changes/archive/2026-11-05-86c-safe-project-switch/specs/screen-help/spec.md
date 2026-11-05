## ADDED Requirements

### Requirement: Project transition refusal feedback

A refused project transition SHALL show the current save or target-read reason without a success notice or a new question.
The picker SHALL remain available for retry or cancellation.
Shared edits and picker entry SHALL use the existing keyboard and MIDI actions.
Picker selection, confirmation and text input SHALL keep their existing keyboard paths.

#### Scenario: A refused transition explains its reason
- **WHEN** an open or create request fails because source saving or target access fails
- **THEN** each request shows the current reason without `Session saved` or a project-open success notice
- **AND** no new mode or question replaces the picker

#### Scenario: Keyboard and MIDI project entry share transition safety
- **WHEN** keyboard and raw MIDI runs make the same edits and open the picker through their existing actions
- **AND** the owner selects or types the same target and confirms with the keyboard
- **THEN** both runs retain the same source work on refusal and save the same source work before success
