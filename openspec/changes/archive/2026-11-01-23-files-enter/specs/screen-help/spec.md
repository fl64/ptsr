## MODIFIED Requirements

### Requirement: First steps hint
The system SHALL, while no pad holds a sample, say on the pads screen how to put one there; short-lived messages (a controller connects, audio is unavailable) do not remove the hint — it is back as soon as the message is gone.

#### Scenario: Empty session
- **WHEN** the program starts with no samples on pads and there is no other message
- **THEN** the hint line says "Pads are empty: F6 opens files, Enter puts a file on the selected pad."

#### Scenario: A message passes
- **WHEN** no pad holds a sample, the program shows "MIDI connected" and the message fades out
- **THEN** the first-steps hint is on the pads screen again

#### Scenario: Done with the hint
- **WHEN** the first pad gets a sample
- **THEN** the hint is not shown anymore
