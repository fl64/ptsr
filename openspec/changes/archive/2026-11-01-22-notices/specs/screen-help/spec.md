## ADDED Requirements

### Requirement: Manual save feedback
The system SHALL show "Session saved" for 5 seconds after a manual save (`Ctrl+S`); the
autosave stays silent, and the hint line comes back when the flash is gone.

#### Scenario: The manual save speaks
- **WHEN** the owner presses `Ctrl+S`
- **THEN** the notice line shows "Session saved" and is back to the hint within 5 seconds

#### Scenario: The autosave is silent
- **WHEN** the session autosaves after an idle period
- **THEN** no message shows; the hints and the screen stay as they were
