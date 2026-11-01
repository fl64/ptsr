## ADDED Requirements

### Requirement: Filter and favorites
The system SHALL let the owner filter the files screen by name and keep favorite folders, fully controlled from the keyboard and a controller.

#### Scenario: Search by name
- **WHEN** the owner types "amen" in the filter
- **THEN** the list narrows to files and folders with "amen" in the name

#### Scenario: Favorite folder
- **WHEN** the owner presses Ctrl+B on a folder, goes elsewhere and restarts the program
- **THEN** the folder is listed first with a ★ in every folder, and Enter on it opens it

#### Scenario: Typing text in the filter
- **WHEN** the owner opens the filter line with the `/` key
- **THEN** all keys, including the pad block, type text in any layout, pads do not sound; Enter keeps the filter and closes the line, Esc clears the filter and closes the line; after the line closes, the block keys play pads again

### Requirement: Preview
The system SHALL play the audio file selected on the files screen right away, apart from the pads, switch to the next file without a click when the selection moves, stop when a folder is selected or the screen is left, and let the owner turn this off and on with Space; it is on by default.

#### Scenario: Scrolling through files
- **WHEN** the owner scrolls the list with the down arrow
- **THEN** only the current file sounds, the previous one stops without a click

#### Scenario: Preview off
- **WHEN** the owner presses Space on the files screen
- **THEN** the preview stops, the help line shows "preview: off", and moving through files stays silent until Space is pressed again

#### Scenario: Fast scrolling through long files
- **WHEN** the owner scrolls past ten long files quickly and stops on one
- **THEN** only the file the selection stopped on is played, without waiting for the skipped ones to load
