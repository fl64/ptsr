## MODIFIED Requirements

### Requirement: Files screen
The system SHALL provide a files screen (F6) with the pad grid of the current bank on the left (what each pad plays, missing files, loading progress) and the folders and audio files of the current folder on the right, with navigation into folders and up, controlled from the keyboard and a controller; the first start opens `~/Music` or, if it does not exist, the home folder. The title shows the folder as a short path with `~` in place of the home folder. The preview of the selected file is drawn three rows tall, so its envelope — quiet and loud parts over time — is clearly readable.

#### Scenario: See where the file goes
- **WHEN** the owner takes a file in hand on the files screen
- **THEN** the file line is highlighted as "in hand", the pad grid next to it is highlighted, and the help line says to hit a pad or press Esc

#### Scenario: Into a folder and back
- **WHEN** the owner selects a folder and presses → or Enter, then presses ←
- **THEN** the screen shows the folder's contents, then the parent folder again with the same folder selected

#### Scenario: Short folder in the title
- **WHEN** the owner is inside `/home/owner/Music/breaks`
- **THEN** the title of the files screen reads `~/Music/breaks`

#### Scenario: Envelope in three rows
- **WHEN** a file with a quiet start and a loud middle is selected
- **THEN** its preview strip is three rows tall and the quiet start is visibly lower than the loud middle
