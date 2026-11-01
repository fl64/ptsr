## ADDED Requirements

### Requirement: The chop screen opens in silence
The system SHALL fade out every sounding pad voice without a click when the chop screen
opens, so the owner hears the slices alone. The slice playback of the chop screen is not a
pad voice and is not touched; pad hits there keep playing slices, as in the listen mode;
leaving the chop screen stops nothing.

#### Scenario: A long pad fades on entry
- **WHEN** a one-shot pad plays a long sample and the owner presses F3
- **THEN** the pad fades out without a click as the chop screen opens

#### Scenario: Slices still play after the hush
- **WHEN** the chop screen opened with the pads hushed and the owner hits a pad
- **THEN** the slice of that pad plays, as in the listen mode
