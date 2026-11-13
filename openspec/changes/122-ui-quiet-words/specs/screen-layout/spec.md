# screen-layout Delta

## ADDED Requirements

### Requirement: Clipping feedback names its event time

The master meter SHALL reserve its existing clip row. It SHALL show no clip text before a clipped output block. When the clip latch is on, it SHALL show `CLIP +HH:MM:SS` with the elapsed time of the latest clipped block since program start. Another clipped block SHALL update the time. The existing clean-hit rule SHALL clear the latch and hide the text. A restart SHALL reset the elapsed time. This UI feedback SHALL NOT add audio-callback work.

#### Scenario: Clip feedback appears only for a real clipped block
- **WHEN** the UI receives silence, a clean hit, and then a clipped output block
- **THEN** the clip row is blank until the clipped block and then shows its session-relative time
- **AND** a later clipped block updates the time without moving the meter rows

#### Scenario: A clean hit hides clip feedback
- **WHEN** a clipped block latches the light and a later pad hit produces clean output
- **THEN** the clip row becomes blank under the existing clean-hit rule
- **AND** a restart starts with a blank row and a new elapsed-time origin

### Requirement: Visible words use one glossary

Screen labels, help, notices, and MIDI learn labels SHALL use `transport`, `pattern`, `pad`, `slice`, `sample`, `bank`, `send`, `FX 1`, and `FX 2` for those concepts. A visible-name change SHALL NOT rename logical action IDs, saved MIDI groups, source IDs, or project fields. Text at 80x26 and 120x40 SHALL keep its safety priority.

The six-cell controller map SHALL use `ptrn` for the pattern screen action. MIDI learn SHALL show the full `pattern` label for that action.

#### Scenario: Controller map names pattern in six cells
- **WHEN** the owner views a controller with the pattern screen action
- **THEN** the controller map says `ptrn` within six cells and MIDI learn says `pattern`
- **AND** the saved action ID remains `screen.pattern`

#### Scenario: Screen words name the same concepts
- **WHEN** the owner opens Pads, Chop, Files, FX, Settings, F1, and MIDI learn at both supported sizes
- **THEN** visible labels use the glossary for the same concepts and required safety text stays readable
- **AND** learned actions and saved groups still resolve to their existing IDs

### Requirement: Long labels end at a word boundary

A label that does not fit its terminal-cell field SHALL remove complete trailing words before `…` when one complete word fits. If its first word is wider than the field, it SHALL clip between graphemes. It SHALL NOT split a wide grapheme, key cap, numeric value, unit, or required safety word. The field SHALL remain within its current bounds.

#### Scenario: Long names stay readable at both sizes
- **WHEN** pad, sample, project, and folder names contain spaces and wide graphemes at 80x26 and 120x40
- **THEN** each clipped label ends at a word boundary when possible and stays within its field
- **AND** an overwide first word clips between complete graphemes without hiding a safety message

## MODIFIED Requirements

### Requirement: Screen columns and titles

The right column SHALL start at the bank row. F2 SHALL show meters and the pad panel. F6 SHALL show its path and filter above the file list. F3 SHALL show slice information. The title SHALL stand above the strip. It SHALL name the held file or error on F6 and the file and slices on F3. The existing playback slots SHALL retain their widths and playback rules. The idle PLAY slot SHALL be blank inside its ten cells. Active PLAY and LISTEN SHALL keep their text and position. Other mode indicators SHALL use shared title badges.

#### Scenario: The folder above its list
- **WHEN** the Files screen is open
- **THEN** the folder path stands above its first list row in the right column
- **AND** mode badges do not move the strip, grid, bank row, or right column

#### Scenario: Idle playback keeps its space without text
- **WHEN** Pads has no sounding pad at 80x26 or 120x40, then a pad starts and stops
- **THEN** the idle PLAY slot is blank and active PLAY appears in the same ten cells
- **AND** the screen title, mode badges, and shared rows do not move

### Requirement: Pattern feedback preserves layout and safety

Pattern feedback SHALL extend the existing transport segment. It SHALL NOT create a fourth badge or move rows. At 80x26 and 120x40, screen identity, clock, questions, active pad mode, and save or audio warnings SHALL keep their existing priority. Compact pattern identity SHALL use `A1`. The idle PLAY slot SHALL be blank inside its reserved ten cells. Active PLAY and LISTEN SHALL keep their slots. The offline catalogue SHALL cover active, stopped, empty, and pending pattern states.

#### Scenario: Pattern titles retain safety and actual playback slots
- **WHEN** pattern feedback competes with long names, ASK, EDIT, placement, recording, or save and audio warnings at either supported size
- **THEN** the existing priority rules keep required feedback readable inside its bounds
- **AND** PLAY and LISTEN retain ten-cell slots, with blank text in idle PLAY, and shared rows stay fixed

### Requirement: Visible Pitch Gate state

The pad panel, popup, and F1 SHALL show the saved Pitch Gate state and its A1-only scope. When Pitch Gate is On but cannot act, they SHALL give the source or pitch reason in text. The panel SHALL use a dedicated Pitch Gate detail row inside its current bounds. At 80 columns, this row MAY say `PG on: needs slice` for a source restriction. The popup and F1 SHALL keep the full `PG on: needs linked slice` text. The compact mode-row summary SHALL appear only when pad pitch differs from zero. The Pads help row SHALL explain Pitch Gate only for a linked slice with Pitch Gate On. Scope, edit pages, and controls SHALL remain. Feedback SHALL NOT claim a cut from a saved switch, running transport, or manual hit. No top badge, shared screen row shift, or new pad mode SHALL appear.

#### Scenario: Pitch Gate summary stays quiet at zero pitch
- **WHEN** the selected pad has zero pitch and the owner toggles Pitch Gate with keyboard or raw MIDI
- **THEN** the compact mode row has no Pitch Gate summary
- **AND** the panel, popup, and F1 still show the saved state and the pitch or source reason

#### Scenario: Pitch Gate feedback preserves parameter pages
- **WHEN** keyboard or raw MIDI toggles Pitch Gate with edit open or closed, hidden or shown pads, and eligible or ineligible sources
- **THEN** the panel, popup, and F1 show the saved state, A1-only scope, applicable controls, and any source or pitch restriction
- **AND** every existing page, knob target, safety indicator, and input route stays available

#### Scenario: Pitch Gate help row appears only where it applies
- **WHEN** the owner selects a whole file or linked slice with Pitch Gate Off
- **AND** the owner selects a linked slice with Pitch Gate On at zero or positive pitch
- **THEN** the help row explains Pitch Gate only for the linked slice with Pitch Gate On
- **AND** the panel shows the Pitch Gate label and F1 gives the reason for every pad

#### Scenario: Pitch Gate reports source and pitch restrictions
- **WHEN** Pitch Gate is On for an empty pad, a whole file, a detached piece, or a linked slice at zero or negative pitch
- **THEN** feedback keeps the saved On value and identifies the source or pitch restriction in text
- **AND** a source or pitch change refreshes the explanation without automatically changing the saved switch

#### Scenario: Pitch Gate scope stays explicit during manual play
- **WHEN** an eligible pad has Pitch Gate On without A1, with stopped transport, or during mixed manual and A1 playback
- **THEN** the panel and F1 identify the A1-only setting and the visible help states that manual hits stay unchanged
- **AND** a manual hit or running transport cannot turn that setting into a claim that the current sound has a cut

#### Scenario: Pitch Gate explanations fit the existing frame
- **WHEN** Pitch Gate states appear at 80x26 and 120x40 with hidden or shown pads, edit open or closed, and safety warnings
- **THEN** the panel and F1 explain the state within their existing bounds
- **AND** the view adds no top badge and preserves the fixed rows, safety priority, and access to existing controls

### Requirement: A1 overview shows absent and empty states

An absent A1 SHALL show `pattern A1 is empty` with the effective record action. An existing empty A1 SHALL show 16 empty pad rows. Viewing either state SHALL NOT create an edit, dirty mark, or Undo entry.

#### Scenario: Overview distinguishes absent and empty A1
- **WHEN** the owner opens the overview without A1 and then with an existing empty A1
- **THEN** absence gives `pattern A1 is empty` and a recording instruction, while the empty pattern shows 16 empty rows
- **AND** viewing either state does not create an edit, dirty mark, or Undo entry
