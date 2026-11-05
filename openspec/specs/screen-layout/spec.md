# screen-layout Specification

## Purpose
Use a shared frame for Pads, Files and Chop. Keep pad visibility, waveform space and startup loading information consistent as the terminal size changes.

## Requirements

### Requirement: One layout for the screens with pads
Pads (F2), Files (F6) and Chop (F3) SHALL share strip and grid size and position: wave above, pads left, mode column right.
The full-width strip SHALL show live output on F2, the selected file on F6, and the sample with cuts on F3.
The bank row SHALL stand directly above the grid on each screen.

#### Scenario: The same frame on every screen
- **WHEN** the owner switches F2 → F6 → F3 in the same terminal
- **THEN** the strip and the pad grid stay at the same rows and columns, and the pads look the same, waves included; only the right column and the content of the strip change

### Requirement: Screen columns and titles

The right column SHALL start at the bank row.
F2 SHALL show meters and pad panel.
F6 SHALL show folder path and filter directly above its file list.
F3 SHALL show slice line and side list.
The screen title SHALL stand above the strip.
It SHALL show the held file or error on F6, and mode, file and slices on F3.

#### Scenario: The folder above its list
- **WHEN** the files screen is open
- **THEN** the path of the folder stands right above the first line of the list, in the list's column, not above the pads

### Requirement: Wave strip height

The strip SHALL take all height left by the grid and pad waves, at least 8 rows.
When space is short, pad waves SHALL shrink first, then the strip down to 3 rows, so a 26-row terminal still fits the whole grid.

#### Scenario: A tall terminal gives the strip the rest
- **WHEN** the terminal is 60 rows high
- **THEN** the pads draw their waves in three rows, the strip takes all the rows left above them, more than 8, and is the same height on F2, F6 and F3

#### Scenario: A low terminal
- **WHEN** the terminal gets lower step by step
- **THEN** the pad waves take fewer rows first, then the strip gets lower down to 3 rows, and the whole grid always fits

### Requirement: Hide the pads
`Ctrl+O` SHALL toggle the pad grid on F2, F6 and F3, with one persistent state shared across restarts.
When hidden, the strip SHALL replace the grid in the left column from the title line down, with one pad line below.
The right column SHALL retain its full height from the title line without truncation.
Chop assign SHALL show the grid while open.

#### Scenario: Hide the pads
- **WHEN** the owner presses `Ctrl+O` on the pads screen
- **THEN** the grid is gone, the strip is taller by the rows of the grid, and the pad line shows the bank row with bank 1 in brackets and then `pad W kick` for the selected pad W

### Requirement: Hidden-grid pad line

The pad line SHALL show every bank with the same fill as above the grid, the open bank in brackets, then the selected pad's name.
It SHALL flash on a hit.
While open-bank pads are held or flash, that bank's glyphs SHALL show only those pads, in white, then return to the fill.
A narrow window SHALL truncate the pad part first.

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
At start, the master wave strip above the pad grid SHALL show a splash instead of the silent wave until the first sound.
It SHALL show `ptsr - PT's SampleR`, build version, loading line and this exact logo:

```
           
  _ _/_ __
 /_// _\/ 
/         
```

With fewer than four strip rows, the splash SHALL show `ptsr` and the loading line instead of full logo, name and version.

#### Scenario: The splash at start
- **WHEN** the owner starts ptsr
- **THEN** the wave strip shows the logo, `ptsr - PT's SampleR`, the version and the
  loading line, and the pads, the help line and the status line show as always

### Requirement: First sound ends the splash

The first sounding keyboard/controller pad note, sample preview or started recording SHALL end the splash permanently and show the wave.
A sound request SHALL end it only when the audio queue accepts it while the project is ready.
No audio, a full queue, an unready project or a silent pad hit SHALL NOT end it.
Keys or elapsed time alone SHALL NOT close it.

#### Scenario: A pad hit during the splash
- **WHEN** the splash shows and the owner hits pad `Q`
- **THEN** pad `Q` plays at once and the strip shows the wave from then on

### Requirement: Splash screen and edit scope

The splash SHALL appear only on Pads, with other screens showing their own content.
Returning to Pads before the first sound SHALL show it again.
Pad edit SHALL replace it with its envelope or filter curve.
Closing edit before the first sound SHALL restore the splash.

#### Scenario: A command key during the splash
- **WHEN** the splash shows and the owner presses F3
- **THEN** the chop screen shows its own content; when the owner returns to the pads screen
  before the first sound, the splash shows again

#### Scenario: The pad edit takes the strip from the splash
- **WHEN** the owner opens the pad edit before the first sound
- **THEN** the strip shows the envelope or filter curve instead of the splash
- **AND WHEN** the owner closes the edit before the first sound
- **THEN** the splash shows again

### Requirement: Splash loading status

The session SHALL load before the first screen.
The loading line SHALL count pads with samples loading, say `ready` after loading, or name a file that failed to load.
Sample loading SHALL continue under the splash, and the rest of the screen SHALL remain as usual.

#### Scenario: A missing sample on the splash
- **WHEN** the session names a sample file that does not exist and the owner starts ptsr
- **THEN** the loading line names that file

### Requirement: Persistent save status

Every screen SHALL show `Unsaved` while changes await saving and `Save failed` after a write refusal.
`Save failed` SHALL take precedence until a save succeeds.
The indicator SHALL persist after the error notice ends and while another notice appears.
A successful save SHALL clear the indicator until another change occurs.

#### Scenario: Pending work has a visible save state
- **WHEN** a keyboard or controller action changes the session before autosave
- **THEN** the status shows `Unsaved` on Pads, Files, Chop, FX and Settings

#### Scenario: A save failure stays visible after its notice
- **WHEN** a save fails and its notice expires or another notice appears
- **THEN** `Save failed` remains visible in the shared status line on every screen, including small and large supported terminals
- **AND** no new mode opens, no row moves, and existing help, notices and controls remain available

#### Scenario: Successful retry clears the save warning
- **WHEN** a manual retry saves all pending work
- **THEN** neither `Unsaved` nor `Save failed` remains in the status
- **AND** a later edit shows `Unsaved` until its save succeeds
