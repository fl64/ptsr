## MODIFIED Requirements

### Requirement: Banks on J and L
The system SHALL turn the banks with `PgUp` `PgDn` and `Ctrl+1`…`8` on every screen with
pads, in an edit mode too; on the pads screen outside the edit mode `J` `L` (and `←` `→`)
turn them as well. The arrows and the letters do not pick a pad, a pad hit does.

#### Scenario: L turns to the next bank
- **WHEN** bank 1 is open on the pads screen and the owner presses `L`
- **THEN** bank 2 is open and the selected pad is the same place in it

#### Scenario: The next bank while editing a pad
- **WHEN** the edit of pad W in bank 1 is open and the owner presses `PgDn`
- **THEN** bank 2 is open, the edit stays open on pad W of bank 2, and `J` `L` still change the value

#### Scenario: The same keys on the files screen
- **WHEN** bank 1 is open on the files screen and the owner presses `PgDn`
- **THEN** bank 2 is shown on the grid, the file list does not move

### Requirement: Source page
The system SHALL put a SOURCE page into the pad edit after TONE and ENV: the source, the
start and the end, on knobs 1–3 and the value keys. The source steps all, slice 1…N, pad.
Moving the start or the end of a pad that plays `all` or a slice makes it a `pad` pad with
its current bounds first; the status says the pad no longer follows the cuts, and undo
links it back. The pad panel shows the rows of all pages at once, TONE, ENV and SOURCE,
so no row hides when the page changes; the page decides only what the knobs turn and
carries their numbers.

#### Scenario: Edit only this fragment
- **WHEN** pad 7 plays slice 3 and the owner opens its edit, walks to SOURCE and moves the start right
- **THEN** pad 7 starts later and says `pad`, pads 6 and 8 do not change, and the status says pad E (pad 7) no longer follows the cuts and `Ctrl+Z` links it back

#### Scenario: Undo links it back
- **WHEN** the owner presses Ctrl+Z after that
- **THEN** pad 7 says `slice 3` again and plays slice 3

#### Scenario: Pick another slice
- **WHEN** a pad plays the whole break and the owner steps its source to `slice 2`
- **THEN** the pad plays slice 2 and follows its cuts

#### Scenario: Every row in sight
- **WHEN** the edit of a pad is open on the ENV page and the owner presses `Tab` until SOURCE
- **THEN** the panel shows volume, pitch, attack, decay, release, source, start and end the whole time, the knob numbers 1–3 move from attack, decay, release to source, start, end
