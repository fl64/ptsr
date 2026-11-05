## MODIFIED Requirements

### Requirement: Manual save feedback

Each manual save request SHALL show `Session saved` after success or a current refusal reason after failure.
The notice SHALL last five seconds before the normal hint returns, unless another notice replaces it.
The default keyboard path SHALL remain Ctrl+S.
A learned controller button for `session.save` SHALL use the same save action and feedback.
A successful autosave SHALL stay silent.
A failed save SHALL also retain its persistent status after the notice ends.

#### Scenario: The manual save speaks
- **WHEN** the owner presses Ctrl+S and the save succeeds
- **THEN** the notice line shows `Session saved` and returns to the hint within five seconds

#### Scenario: The autosave is silent
- **WHEN** the session autosaves successfully after an idle period
- **THEN** no message shows and the hints and screen stay as they were

#### Scenario: Repeated manual refusals each explain the reason
- **WHEN** the owner requests manual saves twice while the write problem or read protection remains
- **THEN** each request shows a current refusal reason without clearing pending changes or showing `Session saved`

#### Scenario: Keyboard and controller save feedback agree
- **WHEN** keyboard and raw MIDI runs request the same save through Ctrl+S and a learned `session.save` button
- **THEN** both runs report the same success, write refusal or read-protection result
- **AND** both runs retain the same pending work and last saved file after failure
