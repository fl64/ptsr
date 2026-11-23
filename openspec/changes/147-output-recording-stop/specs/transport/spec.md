## ADDED Requirements

### Requirement: STOP cancels a Files preview load while saving a take

On Files, STOP SHALL cancel a pending preview load before it saves the output recording.

#### Scenario: Saving the take cancels the pending preview
- **WHEN** A1 is idle, the output recording runs and a Files preview load is pending
- **THEN** STOP saves the recording and cancels the load
- **AND** a late load result does not play the preview
