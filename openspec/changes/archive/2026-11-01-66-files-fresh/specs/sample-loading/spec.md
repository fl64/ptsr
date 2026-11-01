## ADDED Requirements

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
