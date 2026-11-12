# screen-layout Delta

## MODIFIED Requirements

### Requirement: First sound ends the splash

The first sounding keyboard/controller pad note, sequenced pad note, sample preview or started recording SHALL end the splash permanently and show the wave. A manual request SHALL end it only when the audio queue accepts it while the project is ready. Rejected requests, absent audio, unready projects and silent events SHALL NOT end it. Keys or elapsed time alone SHALL NOT close it.

#### Scenario: A pad hit during the splash
- **WHEN** the splash shows and the owner hits pad `Q`
- **THEN** pad `Q` plays at once and the strip shows the wave from then on

#### Scenario: Sounding pattern notes end the splash without selecting a pad
- **WHEN** an accepted current-project pattern note starts while the splash shows
- **THEN** the strip changes to live output without changing the selected pad or input-source identity
- **AND** an empty pattern, silent event or unready project does not close the splash

### Requirement: Existing mode badge labels

Placement labels SHALL be `IN HAND`, `COPY`, `SWAP` or `ASSIGN`. Marking SHALL show `MARK` or `N MARKED`, editing `EDIT` with its page, learning `LEARN` and questions `ASK`. Recording SHALL show `REC` with elapsed time when space permits. Chop toggles SHALL show `LOOP` and `AS PAD`. Mark and assignment badges SHALL retain counts. Inactive states SHALL NOT produce these mode labels. Transport feedback SHALL remain distinct from pad playback. Controller panels remain out of scope.

#### Scenario: Every existing mode has its badge
- **WHEN** each existing placement, marking, edit, learning, question, recording or Chop toggle state is active in a separate fixture
- **THEN** its badge uses the specified label, page, count or elapsed time
- **AND** transport feedback remains separate from pad-playback state without advertising a controller panel

#### Scenario: Overlapping placements remain visible
- **WHEN** more than one of file placement, copy and swap is pending
- **THEN** one compound PadMode badge names the pending placement modes
- **AND** existing keyboard and MIDI action priorities remain unchanged

## ADDED Requirements

### Requirement: Accepted pattern clock feedback

Every screen SHALL identify accepted A1 playback and its loop-relative bar and beat beside the existing transport feedback. It SHALL use one coherent current-project audio snapshot for clock position, pattern presence and length. Transport position SHALL remain distinct when it exceeds the loop length. Pending replacement SHALL NOT show a false accepted length. Without a pattern, no A1 label SHALL appear.

#### Scenario: Pattern title follows the accepted loop across screens
- **WHEN** A1 repeats while the owner switches screens, tempo changes or a replacement waits for acceptance
- **THEN** the title follows accepted loop and transport positions without estimating time in the UI
- **AND** snapshots from an old project cannot show its pattern or loop length

### Requirement: Pattern feedback preserves layout and safety

Pattern feedback SHALL extend the existing transport segment, not create a fourth badge or move rows. At 80 by 26 and 120 by 40, screen identity, clock, questions, active pad mode and save or audio warnings SHALL keep their existing priority. Compact pattern identity SHALL use `A1`. The offline catalogue SHALL cover active, stopped, empty and pending pattern states.

#### Scenario: Pattern titles retain safety and actual playback slots
- **WHEN** pattern feedback competes with long names, ASK, EDIT, placement, recording or save and audio warnings at either supported size
- **THEN** the existing priority rules keep required feedback readable inside its bounds
- **AND** PLAY and LISTEN retain their ten-cell slots and shared rows stay fixed

### Requirement: Visible Pitch Gate state

The selected pad's mode row SHALL show `PG on` or `PG off`, inactive for an ineligible source or pitch. F1 SHALL explain its pattern-only positive-pitch linked-slice rule and effective controls. Applied changes SHALL use the existing value popup with pad identity. No parameter page or knob target SHALL change. Pitch Gate SHALL NOT create a pad-input mode.

#### Scenario: Pitch Gate feedback preserves parameter pages
- **WHEN** keyboard or raw MIDI toggles Pitch Gate with edit open or closed, hidden or shown pads and eligible or ineligible sources
- **THEN** the panel, popup and F1 show the actual saved state and applicable controls
- **AND** every existing page, knob target, safety indicator and input route stays available

### Requirement: Sequenced pad feedback uses actual voices

Current-project sequenced voices SHALL count toward PLAY and pad hit feedback under existing actual-voice rules. Silent, dropped, queued and old-project events SHALL NOT count. Sequenced events SHALL NOT change selection, marks, held-input state, bank or keyboard/controller identity. Preview and effect tails alone SHALL retain their existing exclusions.

#### Scenario: Pattern voices light PLAY without pretending to be owner input
- **WHEN** real callbacks play and end A1 notes with grid hidden or shown and edit open or closed
- **THEN** PLAY follows current pad voices and applicable pad flashes follow actual starts
- **AND** selection, marks, bank, held inputs and KB/controller identity stay unchanged
