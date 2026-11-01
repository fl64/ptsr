## MODIFIED Requirements

### Requirement: Sound observability
The system SHALL show in the status line the audio thread load in percent and a counter of buffer underruns (xruns), and on the pads screen the master output level of each channel: a bar in dB, a peak-hold mark, and an overload light that latches while the output clips and goes out on the next hit that does not clip. The levels come from the mixed output itself, so they match what is heard and what an offline render contains.

#### Scenario: Buffer underrun
- **WHEN** the audio thread did not fill the buffer in time
- **THEN** the xrun counter goes up and is highlighted

#### Scenario: Meters follow the sound
- **WHEN** a pad plays at −6 dB peak and the owner watches the meters
- **THEN** both channel bars rise to −6 dB and fall when the sound ends, and the peak mark stays briefly at the top

#### Scenario: Overload latches
- **WHEN** layered pads push the output to a full-scale sample and then a single quiet pad plays
- **THEN** the overload light comes on with the clip and goes out only after the quiet hit
