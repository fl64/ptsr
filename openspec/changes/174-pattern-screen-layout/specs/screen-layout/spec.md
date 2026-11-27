## MODIFIED Requirements

### Requirement: A1 overview fits bank sections at supported sizes

F5 SHALL open the Pattern screen with the title `F5 PATTERN`. It SHALL take the place of the pad grid and the pad inspector. From top to bottom, it SHALL show the header line, the beat numbers and the pad rows. Below them it SHALL show the velocity lane, the detail line and the setup line. At 80x26 and 120x40, the safety warning, status and help SHALL stay visible. When the rows do not fit, the list SHALL keep the selected pad row visible and show the count of hidden rows.

#### Scenario: Overview keeps 16 rows and safety text visible
- **WHEN** the Pattern screen shows long pad and project names, a safety warning and many used rows at both terminal sizes
- **THEN** the header, the selected pad row, the lane, the detail and setup lines, the warning, status and help stay within the screen
- **AND** names clip without breaking a wide grapheme or changing the pad order

#### Scenario: Overview uses wide terminal space for names and steps
- **WHEN** the Pattern screen opens on a 120x40 terminal with a one-bar pattern at 1/16
- **THEN** the step cells are wider than three terminal cells and extend across the width that the row labels leave
- **AND** the selected full pad name appears in the detail line when its row label clips

#### Scenario: Overview pages align to musical beats
- **WHEN** the owner selects each straight and triplet division on one, two and long loops at both terminal sizes
- **THEN** each page starts on a whole beat and shows one bar, half a bar or one beat as its width allows
- **AND** no page divides a beat or invents a cell beyond the loop

### Requirement: A1 overview uses current audio position

The Pattern screen SHALL use one coherent current-project audio snapshot for its playback position and accepted loop length. A plate SHALL mark the current time column across the shown pad rows. The plate SHALL differ from hit marks by shape and background, not color alone. A missing or old-project snapshot SHALL show no moving plate. A running pattern SHALL show the page with its playhead. A stopped pattern SHALL show the first page.

#### Scenario: Overview playhead follows audio without false motion
- **WHEN** playback crosses a page boundary, then audio feedback becomes missing or stale
- **THEN** the current page and plate follow only valid audio feedback and no UI timer advances them
- **AND** missing or old-project feedback removes the plate without showing an old position

### Requirement: A1 overview shows absent and empty states

An absent or empty pattern SHALL show two lines in the grid area. The first line SHALL name the effective record control, for example `Ctrl+R records what you play`. For an empty pattern, the second line SHALL name the effective step edit control. For an absent pattern, it SHALL name the effective setup control. Viewing either state SHALL NOT create a dirty mark or an Undo entry.

#### Scenario: Overview distinguishes absent and empty A1

- **WHEN** the owner opens the Pattern screen without a pattern and then with an existing empty pattern
- **THEN** both show the effective record control
- **AND** the empty pattern names the step edit control and the absent pattern names the setup control
- **AND** viewing either state does not create an edit, a dirty mark, or an Undo entry

### Requirement: Catalogue includes the A1 overview

The offline screen catalogue SHALL include the Pattern screen at 80x26 and 120x40. It SHALL cover absent, empty, straight, triplet, off-grid, multiple-event, velocity, hidden-hit and safety-warning states. It SHALL also cover the Step Edit states. Generation SHALL NOT open audio or MIDI devices or change projects.

#### Scenario: Catalogue covers the overview without devices
- **WHEN** the offline catalogue runs with the Pattern screen registered
- **THEN** both sizes include its required states and a missing entry fails coverage
- **AND** generation leaves projects and device access untouched

### Requirement: Count in and loop position are visible

Count-in SHALL show `COUNT` and descending digits 4, 3, 2, 1 with each accepted beat. A1 recording SHALL show current bar, beat and loop length. Quantize grid and strength SHALL remain available without opening Settings. Feedback SHALL advance without owner input. On the Pattern screen, the header line SHALL show `COUNT n` during count-in and `● REC` with bar.beat during recording, and the setup line SHALL show the grid and strength.

#### Scenario: Count in becomes recording at tick zero
- **WHEN** stopped recording proceeds through four beats at either supported terminal size
- **THEN** visible count digits follow the audio beats and change to the recording mark at the accepted pattern start
- **AND** position, length, grid and strength remain readable

### Requirement: Pads pattern event strip

F2 Rhythm SHALL show a header, an all-pads row, and the selected pad's row. A strip of four or more rows SHALL keep an output wave row. Extra rows SHALL show other pads with hits in the open bank. The frame and safety status SHALL keep their positions. For an absent pattern the strip SHALL show the hint `F5 pattern`.

#### Scenario: Pattern strip fits the shared frame

- **WHEN** F2 shows Rhythm with absent, empty, or populated A1 at 80x26 and 120x40
- **THEN** its event rows fit, and the outer frame and warnings stay visible
- **AND** Chop and Files draw no event rows

#### Scenario: Rhythm shows other pad rows when space permits

- **WHEN** several pads in the open bank have A1 hits and the Rhythm view has spare rows
- **THEN** the extra rows name their pads and follow the same moving playback marker

### Requirement: Rhythm stays visible with MIDI and the pad grid

At 80x26, the Pads screen SHALL show Rhythm and the full pad grid with SMC-PAD connected. An absent pattern SHALL show the hint `F5 pattern`. An empty or populated A1 SHALL show the header and both event rows. The MIDI footer SHALL preserve the knob roles, the current view and its controls. The notice SHALL remain visible. Pad Edit, recording, and Full Help SHALL take priority over Rhythm.

#### Scenario: Compact Rhythm keeps its rows and MIDI roles
- **WHEN** the owner opens Pads Rhythm at 80x26 with the grid visible and an SMC-PAD connected
- **THEN** Rhythm, all sixteen pads, the notice and both MIDI banks are visible
- **AND** Pad Edit takes the strip until it closes

### Requirement: A1 correction shows independent edit and play positions

The Pattern screen SHALL show a `STEP EDIT` badge in its header line and a distinct edit cursor. The cursor SHALL not follow playback. The audio plate SHALL use a coherent current-project snapshot and never a UI timer. Both markers SHALL differ by shape and background, not only color. The selected pad row SHALL remain visible.

#### Scenario: Edit cursor does not follow playback
- **WHEN** playback crosses the selected cell, page and loop seam during correction
- **THEN** the edit cursor stays on its page and the play position stays separately identified
- **AND** missing or old-project audio feedback shows no false moving plate

### Requirement: A1 correction fits the selected bank and event detail

With the grid visible at 80x26 and 120x40, Step Edit SHALL keep the used rows and the selected pad row of the Pattern screen. The header, status, safety notice and help SHALL stay visible. The detail line SHALL show the selected event index and count, or an empty-cell message. A long sample name SHALL use the same width rule as the Pattern screen view.

#### Scenario: Edit screen keeps names, targets and safety visible
- **WHEN** the owner selects a long-name pad, an empty cell and a multiple-event cell while a safety warning is present
- **THEN** both terminal sizes show the selected pad, cell and event count without clipping the warning
- **AND** the edit cursor, playback plate and hit marks remain distinct without color

### Requirement: Catalogue includes A1 correction states

The offline screen catalogue SHALL cover correction on the Pattern screen at 80x26 and 120x40. It SHALL include empty, straight, triplet, off-grid, multiple-event, independent-cursor and safety states. Generation SHALL not open audio or MIDI devices or edit a project.

#### Scenario: Catalogue covers correction without devices
- **WHEN** the offline catalogue runs with correction states registered
- **THEN** both sizes contain the required states and a missing entry fails coverage
- **AND** generation leaves projects and device access unchanged

### Requirement: Visible words use one glossary

Revised labels SHALL use the glossary terms `transport`, `pattern`, `pad`, `slice`, `sample`, `bank`, `send`, `FX 1`, and `FX 2`. The Pattern screen, the Pads strip and the screens view SHALL name the pattern `pattern`, not `A1`. Existing `CLOCK`, `PG` and `scope pad N` labels SHALL keep their text until later changes. Revised labels SHALL NOT rename action IDs, saved MIDI groups, source IDs, or project fields. Text SHALL keep its safety priority at both supported sizes.

#### Scenario: Screen words name the same concepts
- **WHEN** the owner opens Pads, Pattern, Chop, Files, FX, Settings, F1, and MIDI learn at both supported sizes
- **THEN** revised labels use the glossary terms for the same concepts
- **AND** required safety text stays readable
- **AND** learned actions and saved groups keep their existing IDs

## REMOVED Requirements

### Requirement: Bank sections show identity across the vertical list

**Reason**: The Pattern screen has no bank frames and no fixed bank index. The pad label uses its bank tint.
**Migration**: See "A1 overview shows every bank in one vertical list" in the patterns specification.
