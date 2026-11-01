## MODIFIED Requirements

### Requirement: Files screen
The system SHALL provide a files screen (F6) with the pad grid of the current bank on the left (what each pad plays, missing files, loading progress) and the folders and audio files of the current folder on the right, with navigation into folders and up, controlled from the keyboard and a controller; the first start opens the samples root (default `~/ptsr/samples`); while a project is open the files screen starts in that project's `samples/` folder. `Enter` on a file loads it onto the selected pad right away and closes the files screen; `Ctrl+Enter` takes it in hand; `Enter` on a folder opens the folder.

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
- **WHEN** a project is open and the owner opens the files screen
- **THEN** the screen starts in that project's `samples/` folder

### Requirement: Preview
The system SHALL let the owner play the audio file selected on the files screen with the Space toggle, apart from the pads: while it is on, the file sounds right away, the next file comes in without a click when the selection moves, and the preview stops when a folder is selected or the screen is left; it is off by default.

#### Scenario: Scrolling through files
- **WHEN** the owner scrolls the list with the down arrow while the preview is off (the default)
- **THEN** nothing sounds while the selection moves; when the preview is on, only the current file sounds and the previous one stops without a click

#### Scenario: Preview on
- **WHEN** the owner presses Space on the files screen
- **THEN** the current file sounds, the help line shows "preview: on", and moving through files previews each of them

#### Scenario: Preview off
- **WHEN** the preview is on and the owner presses Space on the files screen
- **THEN** the preview stops, the help line shows "preview: off", and moving through files stays silent until Space is pressed again

#### Scenario: Fast scrolling through long files
- **WHEN** the preview is on and the owner scrolls past ten long files quickly and stops on one
- **THEN** only the file the selection stopped on is played, without waiting for the skipped ones to load
