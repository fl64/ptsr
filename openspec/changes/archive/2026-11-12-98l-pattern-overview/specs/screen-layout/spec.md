## ADDED Requirements

### Requirement: A1 overview fits bank sections at supported sizes

At 80x26 and 120x40, the overview SHALL show all 16 rows of the selected bank, the screen title, message, status and three-row help. Row names SHALL use the available width and clip by terminal-cell width. The grid header SHALL number visible time cells with a gap between adjacent numbers. Each page SHALL show complete beats and use the most whole bars that fit with readable numbers. Beat shading and bar lines SHALL match the Pads strip.

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

### Requirement: Bank sections show identity across the vertical list

The overview SHALL show a fixed index of all eight banks and arrange shown bank sections in one vertical list. The index SHALL label its count as the number of pads with A1 hits in each bank. Existing bank controls SHALL move the viewport to the selected section. Each section SHALL have a border in its existing bank color. Its number and hit count SHALL remain readable without color. The current bank SHALL have a distinct border and stay visible when it has no hits.

#### Scenario: Overview bank borders show identity without color alone
- **WHEN** the owner moves between banks with visible A1 hits
- **THEN** every shown section uses its bank color and number, and the current section has a distinct border
- **AND** the fixed index explains the pad counts and keeps all eight bank numbers visible

### Requirement: A1 overview uses current audio position

The overview SHALL use one coherent current-project audio snapshot for its playback position and accepted A1 length. A plate SHALL mark the current time column across all 16 pad rows. The plate SHALL differ from hit marks by shape and background, not color alone. A missing or old-project snapshot SHALL show no moving plate. A running A1 SHALL show the page with its playhead. A stopped A1 SHALL show the first page.

#### Scenario: Overview playhead follows audio without false motion
- **WHEN** playback crosses a page boundary, then audio feedback becomes missing or stale
- **THEN** the current page and plate follow only valid audio feedback and no UI timer advances them
- **AND** missing or old-project feedback removes the plate without showing an old A1 position

### Requirement: A1 overview shows absent and empty states

An absent A1 SHALL show `No A1` with the effective record action. An existing empty A1 SHALL show 16 empty pad rows. Viewing either state SHALL NOT create an edit, dirty mark or Undo entry.

#### Scenario: Overview distinguishes absent and empty A1
- **WHEN** the owner opens the overview without A1 and then with an existing empty A1
- **THEN** absence gives a recording instruction, while the empty pattern shows 16 empty rows
- **AND** viewing either state does not create an edit, dirty mark or Undo entry

### Requirement: Catalogue includes the A1 overview

The offline screen catalogue SHALL include the A1 overview at 80x26 and 120x40. It SHALL cover absent, empty, straight, triplet, off-grid, multiple-event, hidden-hit and safety-warning states. Generation SHALL NOT open audio or MIDI devices or change projects.

#### Scenario: Catalogue covers the overview without devices
- **WHEN** the offline catalogue runs with the overview registered
- **THEN** both sizes include its required states and a missing entry fails coverage
- **AND** generation leaves projects and device access untouched
