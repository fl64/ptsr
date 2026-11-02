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

### Requirement: The splash in the wave strip
The system SHALL show at start, in the wave strip above the pad grid (the strip that shows
the master output as a wave), a splash instead of the silent wave, and keep it until the
first sound. The splash shows the logo, `ptsr - PT's SampleR`, the version of the build and
a loading line. The logo is this text, character by character:

```
           
  _ _/_ __
 /_// _\/ 
/         
```

The session file is read before the first screen. The loading line names the number of
pads whose samples are loading and says `ready` when loading is complete, or names a file
that did not load. Sample loading continues under the splash, and the rest of the screen
shows as always. When the strip has fewer than four rows, the splash shows `ptsr` and the
loading line instead of the full logo, name and version.

The splash is visible only on the pads screen. Other screens show their own content. If the
owner returns to the pads screen before the first sound, the splash is visible again.
The pad edit shows its envelope or filter curve in place of the splash. When the edit
closes before the first sound, the splash shows again.
The first sound ends the splash, and it never returns: a note that plays (a pad hit from
the keys or from the controller), a sample preview, or a recording that starts. A sound
request ends the splash only when the audio queue accepts it while the project is ready.
No audio, a full queue, an unready project, or a hit on a pad with no sound does not end
it. No key and no time closes the splash by itself; from the first sound on,
the strip shows the wave.

#### Scenario: The splash at start
- **WHEN** the owner starts ptsr
- **THEN** the wave strip shows the logo, `ptsr - PT's SampleR`, the version and the
  loading line, and the pads, the help line and the status line show as always

#### Scenario: A pad hit during the splash
- **WHEN** the splash shows and the owner hits pad `Q`
- **THEN** pad `Q` plays at once and the strip shows the wave from then on

#### Scenario: A command key during the splash
- **WHEN** the splash shows and the owner presses F3
- **THEN** the chop screen shows its own content; when the owner returns to the pads screen
  before the first sound, the splash shows again

#### Scenario: The pad edit takes the strip from the splash
- **WHEN** the owner opens the pad edit before the first sound
- **THEN** the strip shows the envelope or filter curve instead of the splash
- **AND WHEN** the owner closes the edit before the first sound
- **THEN** the splash shows again

#### Scenario: A missing sample on the splash
- **WHEN** the session names a sample file that does not exist and the owner starts ptsr
- **THEN** the loading line names that file
