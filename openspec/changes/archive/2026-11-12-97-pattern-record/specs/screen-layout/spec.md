# screen-layout Delta

## MODIFIED Requirements

### Requirement: Visible Pitch Gate state

The selected pad's mode row SHALL distinguish Off, On for A1 only, and On but inapplicable. When On, source or pitch restrictions SHALL have a text reason, not color alone. The Pads help row SHALL show the Pitch Gate explanation only for a linked slice with Pitch Gate On. Otherwise ordinary help SHALL use that row. The pad change scope, popup, edit pages and effective controls SHALL remain. Feedback SHALL NOT claim a manual cut or a sounding cut from the saved switch, running transport or a manual hit. No top badge, row shift or new pad mode SHALL appear.

#### Scenario: Pitch Gate feedback preserves parameter pages
- **WHEN** keyboard or raw MIDI toggles Pitch Gate with edit open or closed, hidden or shown pads and eligible or ineligible sources
- **THEN** the panel, popup and F1 show the saved state, A1-only scope, applicable controls and any source or pitch restriction
- **AND** every existing page, knob target, safety indicator and input route stays available

#### Scenario: Pitch Gate help row appears only where it applies
- **WHEN** the selected pad is a whole file or a linked slice with Pitch Gate Off, or a linked slice with Pitch Gate On at zero or positive pitch
- **THEN** the help row explains Pitch Gate only for the linked slice with Pitch Gate On
- **AND** the panel shows the Pitch Gate label and F1 gives the reason for every pad

#### Scenario: Pitch Gate reports source and pitch restrictions
- **WHEN** Pitch Gate is On for an empty pad, a whole file, a detached piece or a linked slice at zero or negative pitch
- **THEN** feedback keeps the saved On value and identifies the source or pitch restriction in text
- **AND** a source or pitch change refreshes the explanation without automatically changing the saved switch

#### Scenario: Pitch Gate scope stays explicit during manual play
- **WHEN** an eligible pad has Pitch Gate On without A1, with stopped transport or during mixed manual and A1 playback
- **THEN** the indicator identifies the A1-only setting and the visible help states that manual hits stay unchanged
- **AND** a manual hit or running transport cannot turn that setting into a claim that the current sound has a cut

#### Scenario: Pitch Gate explanations fit the existing frame
- **WHEN** Pitch Gate states appear at 80×26 and 120×40 with hidden or shown pads, edit open or closed and safety warnings
- **THEN** the mode, compact Pitch Gate state and applicable text explanation remain readable in the existing panel and help rows
- **AND** the view adds no top badge and preserves the fixed rows, safety priority and access to existing controls

## ADDED Requirements

### Requirement: Accepted pattern recording indicators

Every current screen SHALL show accepted count-in, `REC A1`, erasure or Finishing state without confusing it with microphone or output capture. Pending requests SHALL NOT claim an accepted phase. Phase, loop position and target SHALL come from one coherent current-project audio snapshot. Stale feedback SHALL NOT restore another project's recording label.

#### Scenario: Recording indicators follow accepted audio state
- **WHEN** a recording command waits, count-in advances, recording starts, erasure toggles and the pass finishes
- **THEN** each screen shows the matching accepted phase and A1 target without premature REC feedback
- **AND** stale or mixed-project feedback cannot change those shown indicators

### Requirement: Count in and loop position are visible

Count-in SHALL show `COUNT` and descending digits 4, 3, 2, 1 with each accepted beat. A1 recording SHALL show current bar, beat and loop length. Quantize grid and strength SHALL remain available without opening Settings. Feedback SHALL advance without owner input. The full step grid and editable pad rows SHALL NOT appear in 97.

#### Scenario: Count in becomes recording at tick zero
- **WHEN** stopped recording proceeds through four beats at either supported terminal size
- **THEN** visible count digits follow the audio beats and change to `REC A1` at the accepted pattern start
- **AND** position, length, grid and strength remain readable without a new step screen

### Requirement: Pattern work keeps the shared frame

Pattern feedback SHALL keep the fixed title, status, strip, grid and three-row help positions. Compact pattern controls SHALL show Bars, Quantize and Strength with a selected row and applicable edit restrictions. Count digits SHALL use available content space without covering questions or text input. Safety indicators SHALL retain their existing width priority.

#### Scenario: Pattern controls fit without moving the pads
- **WHEN** pattern controls open and close on each current screen at 80×26 and 120×40
- **THEN** the selected setting, current values and close controls remain readable
- **AND** screen identity, pad rows, save failures and audio warnings retain their fixed frame

### Requirement: Erasure has a distinct visible pad role

Erasure SHALL show `ERASE` with the A1 recording state. Its held targets SHALL have a distinct erase marker, not PLAY or ordinary held-note styling. The view SHALL identify the target bank and pad even with a hidden grid. Leaving erasure SHALL clear its target markers without a sounding-pad flash. No-release fallback SHALL name its one-loop target duration.

#### Scenario: Erasure remains distinct from sounding pads
- **WHEN** erasure targets pads across a bank change with the grid shown or hidden
- **THEN** ERASE and actual target identities remain visible without implying a manual sounding hit
- **AND** leaving erasure clears those marks and shows the return to ordinary recording

### Requirement: Recording catalogue covers supported states

The offline screen catalogue SHALL cover CountIn, Recording, Erase, Finishing and PatternControls on every applicable registered screen at both supported sizes. It SHALL include busy, blocked and safety-warning combinations. Missing entries SHALL fail the existing coverage check. Generation SHALL NOT open devices or edit projects.

#### Scenario: Catalogue includes recording states and safety combinations
- **WHEN** the offline catalogue generates after 97 implementation
- **THEN** both supported sizes contain every applicable recording phase and controls state
- **AND** the matrix detects missing entries and keeps save or audio warnings visible in crowded fixtures
