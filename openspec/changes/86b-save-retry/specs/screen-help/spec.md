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

## ADDED Requirements

### Requirement: Learn the save action from F12

The F12 MIDI tab SHALL list `session.save` as `save session` under GLOBAL.
The existing learning steps SHALL accept note and CC buttons, require Enter to save, and support Esc to cancel.
After restart, the confirmed assignment SHALL use the same save action and feedback as Ctrl+S on every input screen.
Learning SHALL NOT require a configuration edit or a new factory assignment.

#### Scenario: The save action appears in GLOBAL
- **WHEN** the owner opens the MIDI tab on F12 and opens GLOBAL
- **THEN** the action list contains one `save session` row for a button

#### Scenario: Save learning waits for confirmation
- **WHEN** the owner starts learning `save session` and presses an unused note or CC button
- **THEN** the row shows the pending assignment without changing the saved configuration
- **AND** Esc cancels the pending assignment without saving the session

#### Scenario: The learned save button survives restart
- **WHEN** the owner confirms a pending `save session` button and restarts the program
- **THEN** the saved GLOBAL assignment remains available without a configuration edit
- **AND** the button saves the current session on Pads, Files, Chop, FX and Settings with `Session saved` feedback

#### Scenario: The learned save button retries after write refusal
- **WHEN** a save fails through the button learned from F12 and the owner removes the write problem
- **THEN** pressing that button again saves the pending work and clears `Save failed`
- **AND** the next edit can autosave silently
