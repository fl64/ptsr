# pad-assignment Specification (delta)

## MODIFIED Requirements

### Requirement: Saving the session
The system SHALL save pad samples, modes and volumes of all banks and the last browser folder on every change and on exit, and restore them on the next start; with a project open the session saves into that project's `project.toml` (sample paths inside the project folder relative to it), without a project — into the scratch session file as today.

#### Scenario: Restart
- **WHEN** the owner closes and opens the program again
- **THEN** all pads of all banks sound the same as before closing, and the files screen opens in the same folder

#### Scenario: File is gone
- **WHEN** an assigned file is deleted from disk
- **THEN** the pad is marked as "missing file" and stays silent, the others work

#### Scenario: Saved into the project
- **WHEN** a project is open and the session autosaves
- **THEN** the `project.toml` of that project changes and the scratch session file stays untouched
