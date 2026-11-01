## MODIFIED Requirements

### Requirement: Files screen
The system SHALL provide a files screen (F6) with the pad grid of the current bank on the left (what each pad plays, missing files, loading progress) and the folders and audio files of the current folder on the right, with navigation into folders and up, controlled from the keyboard and a controller; holding the up or down key keeps moving the selection; the first start opens the samples root (default `~/ptsr/samples`); while a project is open the files screen starts in that project's `samples/` folder; while that folder holds no audio files and no folders yet (a new project), the files screen starts in the folder the owner browsed last. `Enter` on a file loads it onto the selected pad right away and closes the files screen; `Ctrl+Enter` takes it in hand; `Enter` on a folder opens the folder.

#### Scenario: Enter puts the file on the selected pad
- **WHEN** a pad is selected and the owner presses `Enter` on a file
- **THEN** the file loads onto that pad, the files screen closes, and the pads screen shows the file on the pad

#### Scenario: See where the file goes
- **WHEN** the owner takes a file in hand with `Ctrl+Enter` on the files screen
- **THEN** the file line is highlighted as "in hand", the pad grid next to it is highlighted, and the help line says to hit a pad or press Esc

#### Scenario: Into a folder and back
- **WHEN** the owner selects a folder and presses → or Enter, then presses ←
- **THEN** the screen shows the folder's contents, then the parent folder again with the same folder selected

#### Scenario: First start in the samples root
- **WHEN** the program starts for the first time and the owner opens the files screen
- **THEN** the screen shows the samples root folder

#### Scenario: The project's samples come first
- **WHEN** a project whose `samples/` holds material is open and the owner opens the files screen
- **THEN** the screen starts in that project's `samples/` folder

#### Scenario: Holding the arrow scrolls the list
- **WHEN** the owner holds the down arrow on the files screen
- **THEN** the selection keeps moving down the list while the key is held

#### Scenario: A new project starts where you browsed
- **WHEN** the owner browsed `~/Music/breaks`, creates the project "beat2" and opens the files screen
- **THEN** the screen shows `~/Music/breaks`, not the empty `beat2/samples/`

### Requirement: Filter and favorites
The system SHALL let the owner filter the files screen by name and keep favorite folders, fully controlled from the keyboard and a controller.

#### Scenario: Search by name
- **WHEN** the owner types "amen" in the filter
- **THEN** the list narrows to files and folders with "amen" in the name

#### Scenario: Favorite folder
- **WHEN** the owner presses Ctrl+B on a folder, goes elsewhere and restarts the program
- **THEN** the folder is listed first with a ★ in every folder, and Enter on it opens it; in the folder that holds it the folder is listed once, with its ★, not twice

#### Scenario: Typing text in the filter
- **WHEN** the owner opens the filter line with the `/` key
- **THEN** all keys, including the pad block, type text in any layout, pads do not sound; Enter keeps the filter and closes the line, Esc clears the filter and closes the line; after the line closes, the block keys play pads again
