# screen-layout Delta

## ADDED Requirements

### Requirement: Pads pattern event strip

F2 SHALL show three A1 event rows below its live output wave, within the shared strip rectangle. The rows SHALL be a header, an all-pads row and the selected pad's row. A strip of four or more rows SHALL keep at least one wave row. A three-row strip SHALL show only the event rows. Title, bank, pad grid, right column, help and safety status SHALL keep their positions. Other screens SHALL keep their strip contents. No A1 SHALL show a recording instruction without creating a pattern.

#### Scenario: Pattern strip fits the shared frame
- **WHEN** F2 shows absent, empty or populated A1 at 80x26 and 120x40, with pads shown or hidden
- **THEN** the event rows fit below the wave or replace it in a three-row strip, and the outer frame and warnings stay
- **AND** Chop and Files draw no event rows, and hidden pads give the strip more wave rows

### Requirement: Pattern strip identifies time and event density

A word header SHALL identify A1, the playback bar and beat, the loop length and the view division. It SHALL also name the selected pad number and name. The all-pads row SHALL mark cells where any pad of the current bank has an onset. Empty, aligned, off-grid and multiple-event cells SHALL have distinguishable forms, with dim empty cells. Same-tick rows SHALL remain visibly multiple. Drawing SHALL NOT change raw rows or settings.

#### Scenario: Pattern strip distinguishes subdivisions and event density
- **WHEN** straight and triplet pages contain aligned hits, between-line hits, collisions, empty cells and hits of other pads in the bank
- **THEN** their forms, beat backgrounds, bar lines, header words and the all-pads row identify the corresponding data
- **AND** drawing changes no raw row, velocity, duration or setting

### Requirement: Pattern strip groups beats symmetrically

Each beat SHALL start with one space. Each cell SHALL take its glyph and one space, so every glyph has a space on both sides. Alternate beats SHALL have a darker background. A vertical line SHALL separate bars, and no line SHALL separate beats. Beat groups SHALL follow the straight or triplet division. The playback plate SHALL cover a glyph and its two neighbor spaces.

#### Scenario: Pattern strip centers glyphs in beat groups
- **WHEN** straight and triplet pages show one or more bars with the playback plate on an onset and on an empty cell
- **THEN** every glyph has one space on each side, alternate beats have a darker background and only bar lines separate groups
- **AND** the plate covers exactly three columns centered on the current glyph

### Requirement: Bank row shows where A1 plays

The bank row SHALL underline the number of each bank that has a visible A1 onset. During playback, each closed bank's pad map SHALL light, in white, the pads with an A1 onset in the current 1/16 playback step. The open bank SHALL keep its current flashes. The row SHALL NOT change the open bank. A missing or stale audio position SHALL light no pattern pad.

#### Scenario: Bank row shows pattern banks and their hits
- **WHEN** A1 has onsets on banks 1 and 3, bank 1 is open, and transport stops, then plays
- **THEN** the row underlines numbers 1 and 3, and during playback bank 3 lights the pads with an onset in the current step
- **AND** the open bank does not change, and a missing or stale position lights no pattern pad

### Requirement: Pattern playback plate follows the audio position

The playback plate SHALL use coherent current-project audio position, never a UI timer. It SHALL be a centered three-column light plate in both event rows, without an arrow, frame or extra row. The page SHALL follow the plate. A missing or stale snapshot, or one from an old project, SHALL show no plate and no motion.

#### Scenario: Pattern playback plate follows the audio position
- **WHEN** playback crosses cells and pages, and then the project epoch changes or the loop length no longer matches
- **THEN** the plate marks the cell of the audio position and its page
- **AND** a stale or mismatched position shows no plate

### Requirement: Pattern strip yields to overlays and keeps safety visible

Pad-parameter curves, the splash and audio recording SHALL keep their use of the strip, and the event rows SHALL NOT draw then. Save, audio and xrun warnings SHALL stay visible with the event rows at both supported sizes. F2 SHALL keep terminal-cell drawing. Only Chop collects kitty images.

#### Scenario: Pattern strip yields to overlays and keeps safety visible
- **WHEN** F2 shows A1 with a pad-parameter curve, the splash, audio recording, or save and audio warnings
- **THEN** curves, the splash and the recording wave take the strip without event rows
- **AND** warnings stay visible next to the event rows at 80x26 and 120x40

### Requirement: Pattern view catalogue states

The offline catalogue SHALL cover the pattern view at both supported sizes. It SHALL include straight and triplet views, off-grid and multiple events, the hidden-grid preference and safety warnings. Missing required entries SHALL fail coverage. Generation SHALL NOT open audio or MIDI devices or edit projects.

#### Scenario: Catalogue covers the pattern view
- **WHEN** the offline catalogue generates
- **THEN** both sizes contain the required pattern-view entries and a missing entry fails coverage
- **AND** generation leaves projects and device access untouched
