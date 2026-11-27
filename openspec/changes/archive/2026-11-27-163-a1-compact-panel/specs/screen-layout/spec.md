## ADDED Requirements

### Requirement: The narrow A1 panel uses the work area

At 80x26, an open A1 command panel SHALL replace the grid below the title and use the full width. The project picker and full Help SHALL have priority over that panel. The confirmation question SHALL stay in front of the panel. Closing the panel SHALL restore the grid.

#### Scenario: Compact A1 panel and picker stay readable
- **WHEN** the owner opens the A1 panel at 80x26, then opens or closes the project picker or full Help
- **THEN** the active view uses the work area and the panel shows all 16 cells when active
- **AND** closing the panel restores the A1 grid

## MODIFIED Requirements

### Requirement: A1 overview fits bank sections at supported sizes

With the grid visible at 80x26 and 120x40, the overview SHALL show all 16 rows of the selected bank. The title, message, status and help SHALL stay visible. Row names SHALL use the available width and clip by terminal-cell width. The grid header SHALL number visible time cells with a gap between adjacent numbers. Each page SHALL show complete beats and use the most whole bars that fit with readable numbers. Beat shading and bar lines SHALL match the Pads strip.

#### Scenario: Overview keeps 16 rows and safety text visible
- **WHEN** the overview shows long pad and project names, a safety warning and both terminal sizes
- **THEN** all 16 pad keys of the selected bank, their event rows, pattern identity, warning, status and help stay within the screen
- **AND** names clip without breaking a wide grapheme or changing the pad order

#### Scenario: Overview uses wide terminal space for names and steps
- **WHEN** the overview opens on a 120x40 terminal with a one-bar A1 and long sample names
- **THEN** row names use at least 40 terminal cells and time cells extend across the remaining width
- **AND** the selected full name appears in the title when its row label clips

#### Scenario: Overview pages align to musical beats
- **WHEN** the owner selects each straight and triplet division on one, two and long A1 loops at both terminal sizes
- **THEN** each page starts on a whole beat and fits the maximum complete bars, half bars or beats allowed by its width
- **AND** no page divides a beat or invents a cell beyond the loop

### Requirement: A1 correction fits the selected bank and event detail

With the grid visible at 80x26 and 120x40, the overview SHALL keep all 16 rows of the selected bank. The bank index, title, status, safety notice and help SHALL stay visible. It SHALL show the selected event index and count, or an empty-cell message. A long sample name SHALL use the same width rule as the read-only overview.

#### Scenario: Edit screen keeps names, targets and safety visible
- **WHEN** the owner selects a long-name pad, an empty cell and a multiple-event cell while a safety warning is present
- **THEN** both terminal sizes show the bank, selected pad, cell and event count without clipping the warning
- **AND** the edit cursor, playback plate and hit marks remain distinct without color

### Requirement: Panel replaces the pad grid without moving the frame

The command panel SHALL use the existing pad-grid rectangle outside narrow A1. It SHALL keep the title, strip, bank row, right column, three-row help and status in their current positions. Cells SHALL use the same physical top-to-bottom order as first-bank pads. Each line of a cell label SHALL fit eight terminal cells. The panel SHALL remain readable with kitty graphics on or off. It SHALL not conceal a question, text line or safety warning.

#### Scenario: Panel grid fits both supported sizes
- **WHEN** the panel shows its main and context pages at 80 by 26 and 120 by 40
- **THEN** all 16 cells, page title, fixed bottom row and safety indicators fit inside their existing areas
- **AND** pad-wave images cover no cell and leaving the panel restores the current wave
