## MODIFIED Requirements

### Requirement: Saving the session

The system SHALL save pad samples, modes and volumes of all banks and the last browser folder after changes and on exit.
The system SHALL restore them on the next start.
An open project SHALL save to its `project.toml`, with internal sample paths relative to the project folder.
Without a project, the system SHALL save to the scratch session file.
The system SHALL clear pending changes only after a successful file replacement.
A write failure SHALL retain pending changes and permit a manual retry.
After a write failure, the system SHALL pause automatic retries, including exit saving, until a manual save succeeds.
A successful manual retry SHALL restore ordinary autosave for later changes.
A failed save SHALL leave the last saved file intact.

#### Scenario: Restart
- **WHEN** the owner closes and opens the program again after successful saving
- **THEN** all pads of all banks sound the same as before closing, and the files screen opens in the same folder

#### Scenario: File is gone
- **WHEN** an assigned file is deleted from disk
- **THEN** the pad is marked as "missing file" and stays silent, the others work

#### Scenario: Saved into the project
- **WHEN** a project is open and the session autosaves
- **THEN** the `project.toml` of that project changes and the scratch session file stays untouched

#### Scenario: Failed autosave retains pending work
- **WHEN** an autosave fails after the owner changes a pad
- **THEN** the pad change remains pending and the last saved file stays byte for byte unchanged
- **AND** later edits retain all pending work without resuming automatic retries

#### Scenario: Manual retry restores autosave
- **WHEN** a write problem ends and the owner requests a manual save after a failed save
- **THEN** the current work saves and the pending-change and write-failure states clear
- **AND** a later edit saves through ordinary autosave

#### Scenario: A failed save does not retry on exit
- **WHEN** the owner exits while a write failure remains unresolved
- **THEN** exit saving does not retry the write or report success

#### Scenario: An occupied temporary path preserves the saved file
- **WHEN** a save cannot safely create or write its temporary file
- **THEN** the save fails without changing the last saved file or another file at the temporary path
