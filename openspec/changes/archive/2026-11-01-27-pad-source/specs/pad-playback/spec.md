## ADDED Requirements

### Requirement: Pad source
The system SHALL give every pad with a sample one source and name it in the pad panel:
`all` plays the whole file, `slice N` plays slice N of the file's cuts and follows them,
`pad` plays the pad's own start and end and follows nothing.

#### Scenario: Slices laid out
- **WHEN** slices 1–8 of a break lie on pads 5–12 and the owner selects pad 7
- **THEN** the pad panel says `break.wav · slice 3`

#### Scenario: The whole file
- **WHEN** a pad plays a whole file and the owner selects it
- **THEN** the pad panel says `break.wav · all`

### Requirement: Source page
The system SHALL put a SOURCE page into the pad edit after TONE and ENV: the source, the
start and the end, on knobs 1–3 and the value keys. The source steps all, slice 1…N, pad.
Moving the start or the end of a pad that plays `all` or a slice makes it a `pad` pad with
its current bounds first; the status says the pad no longer follows the cuts, and undo
links it back.

#### Scenario: Edit only this fragment
- **WHEN** pad 7 plays slice 3 and the owner opens its edit, walks to SOURCE and moves the start right
- **THEN** pad 7 starts later and says `pad`, pads 6 and 8 do not change, and the status says pad E (pad 7) no longer follows the cuts and `Ctrl+Z` links it back

#### Scenario: Undo links it back
- **WHEN** the owner presses Ctrl+Z after that
- **THEN** pad 7 says `slice 3` again and plays slice 3

#### Scenario: Pick another slice
- **WHEN** a pad plays the whole break and the owner steps its source to `slice 2`
- **THEN** the pad plays slice 2 and follows its cuts
