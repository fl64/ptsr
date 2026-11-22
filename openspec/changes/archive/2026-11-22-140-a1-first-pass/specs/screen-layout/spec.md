## MODIFIED Requirements

### Requirement: A1 overview shows absent and empty states

An absent A1 SHALL show `pattern A1 is empty` in its title. The next instruction SHALL show the effective record action when audio is ready. An existing empty A1 SHALL show 16 empty pad rows. Viewing either state SHALL NOT create a dirty mark or an Undo entry.

#### Scenario: Overview distinguishes absent and empty A1

- **WHEN** the owner opens the overview without A1 and then with an existing empty A1
- **THEN** absence shows `pattern A1 is empty`, while the empty pattern shows 16 empty rows
- **AND** each view shows the effective record control when audio can accept a pass
- **AND** viewing either state does not create an edit, a dirty mark, or an Undo entry
