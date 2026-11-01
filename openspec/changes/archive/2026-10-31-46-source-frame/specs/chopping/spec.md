## ADDED Requirements

### Requirement: The source pad stands apart
The system SHALL draw, on the chop screen, the frame of a pad that plays the whole file of
the open chop dashed (`┄` `┆`), in the colour a frame of that pad has anyway; the pads of
slices and the other pads keep solid frames. The wave and the `◆ all` note stay.

#### Scenario: The whole file among its slices
- **WHEN** pad 1-1 plays the whole break, pads 1-2…1-8 play its slices and the chop screen shows bank 1
- **THEN** pad 1-1 has a dashed frame and pads 1-2…1-8 solid ones

#### Scenario: Another file
- **WHEN** a pad of the bank plays the whole of another file
- **THEN** its frame stays solid
