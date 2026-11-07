## MODIFIED Requirements

### Requirement: Sound observability

Settings Audio SHALL show audio thread load in percent and the xrun counter, including zero.
The permanent status SHALL show only nonzero xruns, highlighted, without CPU percentage.
Pads SHALL retain channel meters in dB, peak holds and the overload light.
Meters SHALL use the mixed output. The overload light SHALL latch on clipping and clear on the next non-clipping hit.
Sound processing and load measurements SHALL stay unchanged.

#### Scenario: Buffer underrun
- **WHEN** the audio thread did not fill the buffer in time
- **THEN** the xrun counter goes up and is highlighted

#### Scenario: Meters follow the sound
- **WHEN** a pad plays at −6 dB peak and the owner watches the meters
- **THEN** both channel bars rise to −6 dB and fall when the sound ends, and the peak mark stays briefly at the top

#### Scenario: Overload latches
- **WHEN** layered pads push the output to a full-scale sample and then a single quiet pad plays
- **THEN** the overload light comes on with the clip and goes out only after the quiet hit

#### Scenario: Settings retains audio load diagnostics
- **WHEN** the owner opens Settings Audio at either supported size
- **THEN** the current CPU load and xrun count remain readable below the tabs
- **AND** the permanent status omits CPU and zero xruns without moving settings controls
