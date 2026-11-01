## ADDED Requirements

### Requirement: Files screen
The system SHALL provide a files screen (F6) with the pad grid of the current bank on the left (what each pad plays, missing files, loading progress) and the folders and audio files of the current folder on the right, with navigation into folders and up, controlled from the keyboard and a controller; the first start opens `~/Music` or, if it does not exist, the home folder.

#### Scenario: See where the file goes
- **WHEN** the owner takes a file in hand on the files screen
- **THEN** the file line is highlighted as "in hand", the pad grid next to it is highlighted, and the help line says to hit a pad or press Esc

#### Scenario: Into a folder and back
- **WHEN** the owner selects a folder and presses → or Enter, then presses ←
- **THEN** the screen shows the folder's contents, then the parent folder again with the same folder selected

### Requirement: Formats and background loading
The system SHALL load WAV, FLAC, AIFF, MP3 and OGG in a background thread, resample them to the system sample rate, and never interrupt the sound while loading.

#### Scenario: Loading a long file during play
- **WHEN** the owner loads a 5-minute file while pads are playing
- **THEN** the pads sound without dropouts, the file becomes available after loading, the status shows progress

#### Scenario: Broken file
- **WHEN** a file cannot be decoded
- **THEN** the program shows a clear message, the pad keeps its old sound, and the program keeps working

### Requirement: Replacing a sample while it plays
The system SHALL let a voice that is already playing finish with its old sample when the pad gets a new one, without a click or a dropout.

#### Scenario: New sample on a sounding pad
- **WHEN** a long sample plays on pad Q and the owner puts another file on Q
- **THEN** the old sound plays on, and the next hit on Q plays the new file
