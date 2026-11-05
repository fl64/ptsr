## MODIFIED Requirements

### Requirement: Deleted project folder

The system SHALL keep working when the open project's folder disappears from disk.
It SHALL mark pads with files inside that folder as "missing file" and drop the project name from the window title.
It SHALL refuse saving without recreating the project folder.
The refusal SHALL retain pending work and allow a manual retry if the folder becomes available again.

#### Scenario: Folder deleted while open
- **WHEN** the folder of the open project is deleted while the program runs
- **THEN** the program keeps working, the pads are marked as "missing file", and the title shows no project name

#### Scenario: Saving does not recreate a deleted project
- **WHEN** the project folder disappears before or during a save
- **THEN** saving fails without recreating the folder and pending work remains in memory
- **AND** each manual save request reports the refusal

## ADDED Requirements

### Requirement: Read protection survives manual save requests

A session file that fails to load SHALL remain protected against saving.
This protection SHALL apply to damaged files, unreadable files and files from a newer format version.
Manual save requests SHALL report the protection reason without replacing the file.
A write failure alone SHALL NOT establish read protection.

#### Scenario: Damaged or newer session remains protected
- **WHEN** startup cannot load a damaged or newer project or scratch session and the owner requests manual saves
- **THEN** each request explains the refusal and the protected file remains byte for byte unchanged
- **AND** edits do not clear the protection or pending work
