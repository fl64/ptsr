# sample-loading Specification

## Purpose
Files screen and sample loading: folders and audio files next to the pad grid, background decoding and resampling, samples replaced while they play.

## Requirements

### Requirement: Files screen
The system SHALL provide a files screen (F6) with the pad grid of the current bank on the left (what each pad plays, missing files, loading progress) and the folders and audio files of the current folder on the right, with navigation into folders and up, controlled from the keyboard and a controller; holding the up or down key keeps moving the selection; the first start opens the samples root (default `~/ptsr/samples`); while a project is open the files screen starts in that project's `samples/` folder; while that folder holds no audio files and no folders yet (a new project), the files screen starts in the folder the owner browsed last. `Enter` on a file takes it in hand, and the next pad hit puts it on that pad (`Enter` again puts it on the selected pad); `Ctrl+Enter` loads it onto the selected pad right away and closes the files screen; `Enter` and `Ctrl+Enter` on a folder open the folder.

#### Scenario: Enter takes the file, a hit places it
- **WHEN** the owner presses `Enter` on a file and then hits pad W
- **THEN** the file is in hand after `Enter` and lies on pad W after the hit

#### Scenario: Ctrl+Enter puts the file on the selected pad
- **WHEN** a pad is selected and the owner presses `Ctrl+Enter` on a file
- **THEN** the file loads onto that pad, the files screen closes, and the pads screen shows the file on the pad

#### Scenario: See where the file goes
- **WHEN** the owner takes a file in hand with `Enter` on the files screen
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

### Requirement: Preview
The system SHALL play the audio file selected on the files screen right away, apart from the pads: the next file comes in without a click when the selection moves, and the preview stops when a folder is selected or the screen is left; `Space` turns it off and on; it is on by default.

#### Scenario: Scrolling through files
- **WHEN** the owner scrolls the list with the down arrow while the preview is on (the default)
- **THEN** only the current file sounds, the previous one stops without a click

#### Scenario: Preview off
- **WHEN** the owner presses Space on the files screen
- **THEN** the preview stops, the help line shows "preview: off", and moving through files stays silent until Space is pressed again

#### Scenario: Preview on
- **WHEN** the preview is off and the owner presses Space
- **THEN** the current file sounds, the help line shows "preview: on", and moving through files previews each of them

#### Scenario: Fast scrolling through long files
- **WHEN** the preview is on and the owner scrolls past ten long files quickly and stops on one
- **THEN** only the file the selection stopped on is played, without waiting for the skipped ones to load

### Requirement: Favorite the folder of a file
The system SHALL, on the favorite command with a file selected, mark the folder being
shown as a favorite (or unmark it) and say so in the status, naming it the folder of this
file.

#### Scenario: Ctrl+B on a file
- **WHEN** the owner is in "breaks" with a file selected and presses Ctrl+B
- **THEN** the status says "★ breaks: favorite (the folder of this file)" and one level up
  "breaks" has a star; a second Ctrl+B says "breaks: no longer a favorite (the folder of
  this file)"

### Requirement: Preview playhead
The system SHALL draw, while the preview plays on the files screen, a thin green line over the wave of the file at the place that sounds now, moving with the sound; the line is gone when the preview stops or the file ends.

#### Scenario: The line runs with the sound
- **WHEN** the preview of a 4-second file has played for 2 seconds
- **THEN** the green line stands in the middle of the wave

#### Scenario: No line when silent
- **WHEN** the preview is off, or the file has played to its end
- **THEN** no green line is drawn over the wave

### Requirement: A fresh list on F6
The system SHALL read the open folder again every time the files screen (F6) is entered,
so a file saved there while the program runs is listed; the selection stays on the file
it was on when that file is still there.

#### Scenario: A new file appears
- **WHEN** the files screen shows a folder, the owner goes to F2, a new file is saved in
  that folder, and the owner comes back to F6
- **THEN** the new file is in the list and the selection stays where it was

### Requirement: The filter is seen
The system SHALL show a line over the file list while a filter is on: `filter: <text> ·
Esc clears`; `Esc` on the files screen clears the filter first and shows the whole folder.

#### Scenario: The filter is seen
- **WHEN** the owner types `/sn` and presses `Enter`
- **THEN** the list shows the files with `sn` and the line over it says
  `filter: sn · Esc clears`

#### Scenario: Esc clears the filter
- **WHEN** the filter `sn` is on and the owner presses `Esc` on the files screen
- **THEN** the whole folder is listed and the filter line is gone

### Requirement: The wave names its file
The system SHALL name the file of the wave on the files screen next to its length.

#### Scenario: The wave names its file
- **WHEN** the owner selects `kick.wav` on the files screen
- **THEN** the wave is labelled `kick.wav · 0.33 s`
