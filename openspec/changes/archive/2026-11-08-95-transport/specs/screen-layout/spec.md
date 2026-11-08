# Screen layout delta

## MODIFIED Requirements

### Requirement: Shared title mode badges

Every screen SHALL use shared title composition. Badge colors SHALL remain yellow for PadMode, cyan for On and magenta for Attention. Transport SHALL share the first mode badge, or occupy the first badge when no mode is active. ASK, recording and remaining modes SHALL follow in their existing order. At most three badges SHALL appear, separate from fixed playback slots. Color alone SHALL NOT identify a mode.

#### Scenario: Mode badges follow the active screen state
- **WHEN** keyboard or raw MIDI actions enter and leave existing modes on Pads, Files, Chop, FX and Settings
- **THEN** their title badges follow the actual current state on the next frame
- **AND** leaving a mode removes its badge without moving shared screen rows

#### Scenario: Badges retain identity without color
- **WHEN** title badges appear in a terminal with limited colors
- **THEN** their text still distinguishes mode, editing, learning, questions and recording

### Requirement: Existing mode badge labels

Placement labels SHALL be `IN HAND`, `COPY`, `SWAP` or `ASSIGN`. Marking SHALL show `MARK` or `N MARKED`, editing `EDIT` with its page, learning `LEARN` and questions `ASK`. Recording SHALL show `REC` with elapsed time when space permits. Chop toggles SHALL show `LOOP` and `AS PAD`. Mark and assignment badges SHALL retain counts. Inactive states SHALL NOT produce these mode labels. Transport feedback SHALL remain distinct from pad playback. Patterns and controller panels remain out of scope.

#### Scenario: Every existing mode has its badge
- **WHEN** each existing placement, marking, edit, learning, question, recording or Chop toggle state is active in a separate fixture
- **THEN** its badge uses the specified label, page, count or elapsed time
- **AND** transport feedback remains separate from pad-playback state without advertising patterns or a controller panel

#### Scenario: Overlapping placements remain visible
- **WHEN** more than one of file placement, copy and swap is pending
- **THEN** one compound PadMode badge names the pending placement modes
- **AND** existing keyboard and MIDI action priorities remain unchanged

### Requirement: Badge width and compatibility

At supported widths, badges SHALL retain screen identity, transport feedback, questions and the active PadMode label. Compact labels SHALL precede removal of lower-priority badges. File details SHALL yield space first. Crowded Chop headers MAY use `#i/n` for variants and `Nsl` for slice counts without losing these numbers. PLAY and LISTEN SHALL retain their ten-cell slots, styles, actual-playback semantics and stable following columns. Badges SHALL NOT change sound, keys, data or input order.

#### Scenario: Narrow titles retain the pad mode
- **WHEN** several badges compete for title width at 80×26
- **THEN** the screen and current pad mode remain readable, with lower-priority badges omitted
- **AND** no text crosses the title boundary

#### Scenario: Playback slots remain stable beside modes
- **WHEN** PLAY or LISTEN changes between actual playback and silence with optional badges present
- **THEN** each playback slot retains its ten-cell width and existing active or idle style
- **AND** following title content stays in the same columns during that playback change
