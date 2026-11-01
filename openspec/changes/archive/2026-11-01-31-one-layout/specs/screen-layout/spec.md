## ADDED Requirements

### Requirement: One layout for the screens with pads
The system SHALL draw the pads screen (F2), the files screen (F6) and the chop screen (F3) in one frame: a wave strip above the pad grid, the pad grid on the left, the column of the mode on the right. The strip and the grid have the same size and place on all three screens; the strip takes all the height the grid with its pad waves leaves, at least 8 rows. When rows are short, the pad waves give way first, then the strip shrinks down to 3 rows, so a 26-row terminal still fits the whole grid. The strip shows the live output on F2, the selected file on F6, the sample with its cuts on F3; the strip spans the full width on every screen. The bank row stands right above the grid on every screen. The right column starts on the row of the bank row and shows the level meters and the pad panel on F2, the folder path with the filter line right above the list on F6, the slice line and the side list on F3. The title line of the screen stands above the strip and carries what the screen says about itself: the file in hand or the error on F6, the mode, the file and the slices on F3.

#### Scenario: The same frame on every screen
- **WHEN** the owner switches F2 → F6 → F3 in the same terminal
- **THEN** the strip and the pad grid stay at the same rows and columns, and the pads look the same, waves included; only the right column and the content of the strip change

#### Scenario: The folder above its list
- **WHEN** the files screen is open
- **THEN** the path of the folder stands right above the first line of the list, in the list's column, not above the pads

#### Scenario: A tall terminal gives the strip the rest
- **WHEN** the terminal is 60 rows high
- **THEN** the pads draw their waves in three rows, the strip takes all the rows left above them, more than 8, and is the same height on F2, F6 and F3

#### Scenario: A low terminal
- **WHEN** the terminal gets lower step by step
- **THEN** the pad waves take fewer rows first, then the strip gets lower down to 3 rows, and the whole grid always fits
