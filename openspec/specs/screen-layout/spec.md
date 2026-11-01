# screen-layout Specification

## Purpose
TBD - created by archiving change 31-one-layout. Update Purpose after archive.

## Requirements

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

### Requirement: Hide the pads
The system SHALL hide and show the pad grid with `Ctrl+O` on the pads (F2), files (F6) and chop (F3) screens, one state for all three. Hidden, the strip takes the place of the grid: the left column from the title line down, with one pad line under it — the bank row of every bank with its fill as above the grid, the open one in brackets, then the selected pad with its name, flashing on a hit; while pads of the open bank are held or flash, its glyphs in the bank row show only those pads, in white, and the fill again after; a narrow window cuts the pad part first; the right column keeps the full height from the title line down, so nothing of it is cut. The assign mode of the chop screen shows the grid while it is open. The state survives a restart.

#### Scenario: Hide the pads
- **WHEN** the owner presses `Ctrl+O` on the pads screen
- **THEN** the grid is gone, the strip is taller by the rows of the grid, and the pad line shows the bank row with bank 1 in brackets and then `pad W kick` for the selected pad W

#### Scenario: Every bank in the pad line
- **WHEN** the pads are hidden, bank 1 is open and bank 3 holds sounds
- **THEN** the pad line shows every bank with its number, bank 3 with its fill and bank 1
  in brackets, each in its color, the same as the bank row above the grid

#### Scenario: A hit lights in the bank row
- **WHEN** the pads are hidden and the owner holds pad Q of bank 3
- **THEN** while pad Q is held or flashes, the glyphs of bank 3 in the bank row show only
  pad Q, in white, no other pad of the bank; after that they show the fill again

#### Scenario: The right column keeps its rows
- **WHEN** the pads are hidden
- **THEN** the right column shows all of it as with the pads shown: the L and R meters and the overload light with the pad panel on F2, the path and the list on F6, the slice line and the side list on F3

#### Scenario: The same on every screen
- **WHEN** the pads are hidden and the owner switches to F6 and F3
- **THEN** both screens draw without the grid, with the same pad line and the same strip height

#### Scenario: The pad line flashes
- **WHEN** the pads are hidden and the owner hits pad Q
- **THEN** the pad line names pad Q and flashes like a hit pad

#### Scenario: The plan shows the grid
- **WHEN** the pads are hidden and the owner opens the assign mode on the chop screen
- **THEN** the grid is drawn with the plan while the assign mode is open, and is hidden again when it closes

#### Scenario: The choice survives a restart
- **WHEN** the owner hides the pads and restarts the program
- **THEN** the pads are hidden; `Ctrl+O` shows them, and a restart keeps them shown
