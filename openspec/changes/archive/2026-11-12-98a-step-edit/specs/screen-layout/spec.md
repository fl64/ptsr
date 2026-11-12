# Spec Delta

## ADDED Requirements

### Requirement: A1 correction shows independent edit and play positions

The overview SHALL show a `STEP EDIT` badge and a distinct edit cursor. The cursor SHALL not follow playback. The audio plate SHALL use a coherent current-project snapshot and never a UI timer. Both markers SHALL differ by shape and background, not only color. The selected bank and pad SHALL remain visible.

#### Scenario: Edit cursor does not follow playback
- **WHEN** playback crosses the selected cell, page and loop seam during correction
- **THEN** the edit cursor stays on its page and the play position stays separately identified
- **AND** missing or old-project audio feedback shows no false moving plate

### Requirement: A1 correction fits the selected bank and event detail

At 80x26 and 120x40, the overview SHALL keep all 16 rows of the selected bank, bank index, title, status, safety notice and three-row help. It SHALL show the selected event index and count, or an empty-cell message. A long sample name SHALL use the same width rule as the read-only overview.

#### Scenario: Edit screen keeps names, targets and safety visible
- **WHEN** the owner selects a long-name pad, an empty cell and a multiple-event cell while a safety warning is present
- **THEN** both terminal sizes show the bank, selected pad, cell and event count without clipping the warning
- **AND** the edit cursor, playback plate and hit marks remain distinct without color

### Requirement: Catalogue includes A1 correction states

The offline screen catalogue SHALL cover correction at 80x26 and 120x40. It SHALL include empty, straight, triplet, off-grid, multiple-event, independent-cursor and safety states. Generation SHALL not open audio or MIDI devices or edit a project.

#### Scenario: Catalogue covers correction without devices
- **WHEN** the offline catalogue runs with correction states registered
- **THEN** both sizes contain the required states and a missing entry fails coverage
- **AND** generation leaves projects and device access unchanged
