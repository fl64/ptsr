## ADDED Requirements

### Requirement: Help pagination follows the visible area

Help and Messages SHALL use the rows inside the current frame for a page press and for the scroll limit. The Controller list SHALL use the same frame height. The size SHALL account for the MIDI footer, a recording caption, and a terminal resize.

#### Scenario: Help pages follow the visible rows with MIDI
- **WHEN** the owner pages through Help with the MIDI footer visible
- **THEN** the next page starts after the last visible row unless it reaches the end of the list
- **AND** a page back uses the same visible page height
