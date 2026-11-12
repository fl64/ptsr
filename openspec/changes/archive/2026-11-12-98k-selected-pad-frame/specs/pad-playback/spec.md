## ADDED Requirements

### Requirement: Selected pad keeps its frame

The selected pad SHALL keep its cyan frame while it flashes, while it is held and while it is marked.
A flash SHALL fill only its inside with white. The frame cells SHALL keep the panel background.
A marked and selected pad SHALL keep its dark yellow fill. A pad in hand SHALL keep its magenta frame.
Other pads SHALL flash and show marks as before.

#### Scenario: Selected pad keeps its frame on a hit
- **WHEN** A1 plays and a hit of the selected pad flashes, and another pad flashes at the same time
- **THEN** the selected pad has a cyan frame on the panel background and a white inside
- **AND** the other pad has a white frame and a white inside

#### Scenario: Selected marked pad keeps its frame
- **WHEN** marking is on and the selected pad is marked
- **THEN** the selected pad has a cyan frame and a dark yellow inside, and another marked pad has a yellow frame
