# patterns Delta

## ADDED Requirements

### Requirement: Pattern view is independent of playback quantize

The view SHALL support 1/4, 1/8, 1/8T, 1/16, 1/16T, 1/32, 1/32T and 1/64, with 1/16 as the default. View steps SHALL follow this order of cell size, and each finer step SHALL have a smaller cell. Playback Off SHALL NOT remove view cells. View changes SHALL preserve raw rows, quantize, strength, velocities, Gate lengths, dirty state and Undo history. Viewing absent A1 SHALL NOT create it.

#### Scenario: Pattern view divisions preserve the take
- **WHEN** each view division shows patterns at every playback grid and strengths 0, 50 and 100
- **THEN** raw rows and playback settings remain unchanged and Off still has a readable view
- **AND** changing the view or inspecting absent A1 creates no edit or history entry

### Requirement: Pattern cells retain source event identity

Each visible raw row SHALL occupy the half-open view cell containing its compiled playback tick. Hidden rows SHALL remain excluded before quantize. A visible seam-wrapped row SHALL appear at loop start. Off-grid positions and multiple source rows SHALL remain distinguishable, including quantize collisions. The view SHALL NOT merge raw rows.

#### Scenario: Pattern cells show off-grid events and collisions
- **WHEN** off-grid same-pad rows share a cell or compile onto one tick
- **THEN** the cell reports their multiplicity and keeps each raw row in playback-tick, raw-tick and original-row order
- **AND** no raw row changes merely because the view groups it

#### Scenario: Pattern view preserves hidden rows and the loop seam
- **WHEN** a loop shortens with hidden rows and a visible last-tick row quantizes to the seam
- **THEN** only the visible seam row appears at loop start and hidden rows remain saved but absent from cells
- **AND** growing the loop restores the hidden rows at their corresponding positions

### Requirement: Pattern pages preserve musical position

A page SHALL contain the most whole bars that fit the strip width, up to the whole loop, starting at a bar line. If one bar does not fit, a page SHALL contain half a bar, then one beat. A page SHALL NOT split a beat. A running pattern SHALL show the page that holds the playback position. A stopped transport SHALL show the first page.

#### Scenario: Pattern pages cover straight and triplet bars
- **WHEN** the owner views every page of each view across one-bar, two-bar and 64-bar patterns at 80 and 120 columns
- **THEN** every cell appears once in order with complete beats, and a two-bar 1/16 loop fits one page at both widths
- **AND** the page follows the playback position, and a stopped transport shows the first page
