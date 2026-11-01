## MODIFIED Requirements

### Requirement: First steps hint
The system SHALL, while no pad holds a sample, say on the pads screen how to put one there; short-lived messages (a controller connects, audio is unavailable) do not remove the hint — it is back as soon as the message is gone.

#### Scenario: Empty session
- **WHEN** the program starts with no samples on pads and there is no other message
- **THEN** the hint line says "Pads are empty: F6 opens files, Enter takes one, a pad hit puts it there."

#### Scenario: A message passes
- **WHEN** no pad holds a sample, the program shows "MIDI connected" and the message fades out
- **THEN** the first-steps hint is on the pads screen again

#### Scenario: Done with the hint
- **WHEN** the first pad gets a sample
- **THEN** the hint is not shown anymore

### Requirement: Screens group in key order
The system SHALL list the screen keys of the help line in ascending key order: help, chop, files, settings.

#### Scenario: The last group of the help line
- **WHEN** any screen shows its help line
- **THEN** the screens group reads F1, F3, F6, F12 in that order
