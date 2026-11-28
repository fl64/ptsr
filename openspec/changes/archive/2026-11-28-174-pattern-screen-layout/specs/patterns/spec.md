## ADDED Requirements

### Requirement: Pattern screen pages hold whole beats

On the Pattern screen, a page SHALL show one bar when that bar fits with cells of three terminal cells.
If one bar does not fit, a page SHALL show half a bar, then one beat.
A page SHALL NOT split a beat. Straight and triplet views SHALL use whole beat groups.

#### Scenario: Pattern screen pages hold whole beats
- **WHEN** the owner views every page of 1/16, 1/32, 1/64 and triplet views at 80x26 and 120x40
- **THEN** a 1/16 page shows one bar, and finer views show half a bar or one beat when one bar does not fit
- **AND** every cell appears once in order and no page splits a beat

### Requirement: Velocity lane shows the selected row

A velocity lane under the Pattern screen grid SHALL show the highest velocity of each cell of the selected pad row.
It SHALL use a bar from `▁` to `█`. An empty cell SHALL show no bar.

#### Scenario: Velocity lane follows the selected row
- **WHEN** the owner selects pads with soft, loud and no hits on the visible page
- **THEN** the lane shows a low bar, a high bar and no bar under the matching cells
- **AND** a change of the selected pad changes only the lane, not the pattern

### Requirement: Detail line names the selected cell

A detail line under the Pattern screen grid SHALL name the selected step as bar.step, the selected pad and the cell content.
The content SHALL be `no hit`, `soft`, `medium` or `loud` with the velocity, `off grid` with the tick offset, or the number of hits.
At 80x26 and 120x40, the line SHALL fit or end with `…` at a word boundary.

#### Scenario: Detail line names the cell content
- **WHEN** the owner selects an empty cell, a loud hit, an off-grid hit and a cell with two hits
- **THEN** the detail line shows `no hit`, `loud (vel 127)`, `off grid` with its tick offset and `2 hits`
- **AND** the selection does not change the pattern

### Requirement: Setup line shows the pattern settings

A setup line on the Pattern screen SHALL show bars, tempo, view grid, record quantize and strength, metronome and `count-in 1 bar`.
In this change, the line SHALL only show values. The pattern setup controls SHALL keep their current routes.
At 80x26 and 120x40, the line SHALL fit or end with `…` at a word boundary.

#### Scenario: Setup line shows the current settings
- **WHEN** the owner changes bars, quantize or tempo through the current setup routes from the keyboard and the SMC-PAD
- **THEN** the setup line shows the new values and `count-in 1 bar`
- **AND** the setup line itself accepts no input

## MODIFIED Requirements

### Requirement: Pattern pages preserve musical position

On the Pads strip, a page SHALL contain the most whole bars that fit the strip width, up to the whole loop, starting at a bar line. If one bar does not fit, a page SHALL contain half a bar, then one beat. The Pattern screen SHALL use its own page rule. A page SHALL NOT split a beat. A running pattern SHALL show the page that holds the playback position. A stopped transport SHALL show the first page.

#### Scenario: Pattern pages cover straight and triplet bars
- **WHEN** the owner views every page of each view across one-bar, two-bar and 64-bar patterns at 80 and 120 columns
- **THEN** every cell appears once in order with complete beats, and a two-bar 1/16 loop fits one page at both widths on the Pads strip
- **AND** the page follows the playback position, and a stopped transport shows the first page

### Requirement: A1 overview shows every bank in one vertical list

The Pattern screen SHALL show the pads with visible hits and the selected pad in one vertical list, in bank order and pad order. It SHALL NOT draw a frame for each bank. Each row SHALL show the pad key, the pad name in its bank tint, and the visible events at the selected view division. A thin rule SHALL separate two banks only when rows of two or more banks show. The view SHALL use compiled playback ticks without changing saved raw hits. It SHALL keep hidden raw hits hidden and show visible seam hits at the loop start.

#### Scenario: Overview shows occupied and selected pad rows
- **WHEN** the pattern has hits on pads of two banks, including off-grid hits, two source hits in one cell, and a selected empty pad
- **THEN** the list shows only pads with visible hits and the selected pad in bank order, with a rule between the two banks
- **AND** each occupied cell shows its event type and count without merging source hits or changing the pattern

#### Scenario: Overview shows all pad rows and source events
- **WHEN** the owner uses Alt+H with hits on several pads and banks
- **THEN** the selected pad's bank shows all 16 pad rows in order, including empty rows, with the used rows of other banks
- **AND** occupied cells show their event type and count without changing the pattern

#### Scenario: Overview filters hidden hits before seam wrapping
- **WHEN** the pattern has a hidden row beyond its shortened length and a visible row that quantizes to the seam
- **THEN** the hidden row does not appear in any cell or row list
- **AND** the visible seam row appears once at the loop start without changing the raw rows

### Requirement: Overview follows the shared view division and bank

The Pattern screen SHALL use the same runtime view division as the Pads strip. Existing bank and pad selection controls SHALL keep the selected pad row visible. Changing the division, bank or selected pad SHALL change only the projection and selection. It SHALL NOT change playback, quantize, raw hits, history, or dirty state. The view division SHALL return to 1/16 after a project change, as on Pads.

#### Scenario: Overview bank and view changes leave A1 unchanged
- **WHEN** the owner switches banks and view divisions while the pattern plays, including a bank with no visible hits
- **THEN** the selected pad row appears with the selected division, and the pattern keeps its timing
- **AND** raw rows, quantize, history, and dirty state stay unchanged

### Requirement: Overview can hide empty bank sections

By default, the Pattern screen SHALL hide empty pad rows except the selected pad. The saved `pattern.overview_empty_toggle` action (`Alt+H` by default) SHALL add all 16 rows of the selected pad's bank. Use it again to restore the list of used rows. A bank with only hidden raw hits SHALL count as empty. This runtime toggle SHALL NOT change the pattern, the bank or pad selection.

#### Scenario: Overview hides empty banks without losing access
- **WHEN** the pattern has visible hits in two banks and hidden raw hits in another bank
- **THEN** the list starts with used and selected rows only, and a bank with only hidden hits shows no row
- **AND** Alt+H adds all rows of the selected bank, then restores the used rows without changing the pattern

#### Scenario: Overview bank navigation reaches a hidden empty bank
- **WHEN** the owner selects a pad in a bank with no visible hits while empty rows are hidden
- **THEN** the selected pad row appears in bank order
- **AND** another pad selection restores the ordinary list of used rows

### Requirement: Overview step cells keep fixed width

On the Pattern screen, all step cells on one page SHALL have the same width of at least three terminal cells. The cells SHALL share the width that the row labels leave and grow with the terminal. Unused columns SHALL stay at the right end of the grid. The header row SHALL number the beats over their first cells and show `·` over the other cells. The screen SHALL keep beat shading, bar borders, the edit cursor, and the live playback plate.

#### Scenario: Overview fixed cells align every division
- **WHEN** the owner views each straight and triplet division at 80x26 and 120x40
- **THEN** all cells on a page have one width of three or more terminal cells and each beat number stays over its first cell
- **AND** bar, beat, cursor, and moving playback marks remain visible without changing the pattern

### Requirement: Overview shows three velocity levels

An occupied cell SHALL show the level of its highest visible velocity: 1–42, 43–84, and 85–127. A single on-grid hit SHALL use `▪`, `▓` or `█`, and brightness SHALL follow the same level. A single off-grid hit SHALL retain `~`. Several hits SHALL retain their count mark. The detail line SHALL show the selected event's numeric velocity. Brightness SHALL NOT be the only mark that identifies a hit.

#### Scenario: Overview velocity levels keep event marks
- **WHEN** visible hits have velocities 1, 42, 43, 84, 85, and 127, including off-grid and multiple-hit cells
- **THEN** the three glyphs and brightness levels follow those boundaries and the off-grid and count marks stay
- **AND** the selected event shows its numeric velocity without changing raw rows

### Requirement: A1 shows the selected position and recording settings

The Pattern screen SHALL show one header line. It SHALL show the screen title, the transport state, the position as bar.beat and the loop length. It SHALL also show the tempo, the bar page and the badge of the active mode. The selected time column SHALL have a mark distinct from the playback mark. The keyboard step actions and SMC-PAD STEP SHALL move the selected column. At 80x26 and 120x40, the header line SHALL fit.

#### Scenario: A1 position remains visible across a bar line
- **WHEN** the owner uses a keyboard step action or turns SMC-PAD STEP across a bar line
- **THEN** the marked time column and the detail line move to the next cell
- **AND** the header, the view grid and the recording quantize remain readable at both supported sizes

#### Scenario: A1 selection stays visible while playback passes another bar
- **WHEN** the owner selects a time cell and playback moves beyond that page
- **THEN** the selected column and visible page remain at the chosen time
- **AND** the header shows the playback position separately

### Requirement: Cancel count in without recording

A second recording toggle or an unclaimed Esc SHALL cancel count-in and leave transport stopped. On the Pattern screen, Esc and the SMC-PAD command P1 SHALL cancel count-in and keep the Pattern screen, bank and selected pad. Help and visible questions SHALL receive Back first. A newly created empty A1 SHALL remain. Cancellation SHALL add no recorded rows or pass Undo step. Releases from warm-up pads SHALL retain their original sound ownership.

#### Scenario: Cancel count in with a pad held
- **WHEN** a pad sounds during count-in and the owner cancels before tick zero and releases that pad
- **THEN** no recorded hit or pass Undo step appears and transport stays stopped
- **AND** the release reaches its original pad without recording a late event

#### Scenario: Back cancels count-in on A1
- **WHEN** the owner presses Esc or command P1 during count-in on the Pattern screen
- **THEN** count-in stops, the transport stays stopped and the Pattern screen stays open at the same bank and pad
- **AND** if Help or a question covers the Pattern screen, the first Back closes that window and count-in continues

### Requirement: A1 step correction needs an explicit mode

The owner SHALL enter correction only from the Pattern screen with an existing A1.
Entry and exit SHALL leave transport, raw hits, dirty state and history unchanged.
A conflicting input owner or pending capture SHALL refuse entry without ending its work.

#### Scenario: Step correction entry keeps A1 playing
- **WHEN** the owner requests correction with absent, empty, stopped and playing A1
- **THEN** absent A1 gives the effective record action, while existing A1 enters without moving transport
- **AND** entry uses the chosen cell, or the playback or first cell before the owner chooses a position

#### Scenario: Step correction refuses conflicting work
- **WHEN** capture, recovery, placement, marking, pad edit, pattern controls, audio recording or a question owns input
- **THEN** correction refuses with a next action and does not change the existing work
- **AND** text, pickers, learning and original pad releases keep their priority

### Requirement: A1 keeps the selected position during correction

Entry to Step Edit SHALL use the selected position of the Pattern screen when the owner selects a cell.
F7, Esc, and command pad 1 SHALL return to the Pattern screen view with the last edit position.
The existing input owners SHALL retain their priority.
A pad edit result SHALL show the target pad, the global step, and the action.

#### Scenario: A1 keeps the chosen step through edit and Undo
- **WHEN** the owner selects a later step, enters Step Edit, adds a hit, then uses Undo and Back
- **THEN** the pad edit reports its pad, step, and result
- **AND** Undo restores the hit state and the Pattern screen keeps the edited step
