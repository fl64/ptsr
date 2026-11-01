# sample-loading Specification (delta)

## MODIFIED Requirements

### Requirement: Files screen
The system SHALL provide a files screen (F6) with the pad grid of the current bank on the left (what each pad plays, missing files, loading progress) and the folders and audio files of the current folder on the right, with navigation into folders and up, controlled from the keyboard and a controller; the first start opens the samples root (default `~/ptsr/samples`); while a project is open the files screen starts in that project's `samples/` folder.

#### Scenario: See where the file goes
- **WHEN** the owner takes a file in hand on the files screen
- **THEN** the file line is highlighted as "in hand", the pad grid next to it is highlighted, and the help line says to hit a pad or press Esc

#### Scenario: Into a folder and back
- **WHEN** the owner selects a folder and presses → or Enter, then presses ←
- **THEN** the screen shows the folder's contents, then the parent folder again with the same folder selected

#### Scenario: First start in the samples root
- **WHEN** the program starts for the first time and the owner opens the files screen
- **THEN** the screen shows the samples root folder

#### Scenario: The project's samples come first
- **WHEN** a project is open and the owner opens the files screen
- **THEN** the screen starts in that project's `samples/` folder
