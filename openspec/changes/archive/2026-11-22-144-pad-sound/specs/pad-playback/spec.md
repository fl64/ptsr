## ADDED Requirements

### Requirement: Each pad has a stereo position and fine tune

Each pad SHALL support Pan from minus 100 to plus 100 percent and Fine tune from minus 100 to plus 100 cents. For mono, use a constant power arc with unity per channel at center. For stereo, use balance. Glide changes and take the shared effect sends after pan. Tune cents relative to the current semitone setting.

#### Scenario: Pan moves a mono pad and its sends together

- **WHEN** a mono pad pans from left through center to right with both sends active
- **THEN** the center preserves the prior stereo level
- **AND** each end sends the pad only to the corresponding side before the shared effects

#### Scenario: Stereo balance preserves the nearer channel

- **WHEN** a stereo pad plays with pan toward one side
- **THEN** the opposite side attenuates and the nearer side keeps its level

#### Scenario: Fine tune spans one semitone in each direction

- **WHEN** the owner changes Fine tune to either limit while a pad sounds
- **THEN** pitch reaches one semitone from the coarse setting with a smooth change
- **AND** the page shows semitones and cents

### Requirement: A pad shapes its envelope

The pad SHALL have linear or exponential decay and Gate release. Hold SHALL run from zero to 2000 ms. Linear with zero hold SHALL keep the fade at the source end. With a hold or the exponential curve, a positive decay SHALL start after attack and hold in output time. The attack SHALL stay linear. The exponential stages SHALL reach minus 60 dB at their set duration. They SHALL then fade to zero within 2 ms. The source end, Panic, voice steals and mute groups SHALL keep their stop conditions.

#### Scenario: Hold delays the selected decay curve

- **WHEN** a long pad plays with attack, hold and decay
- **THEN** the decay starts at the end of hold
- **AND** the linear and exponential results reach their defined endpoints

#### Scenario: Gate release preserves Panic and group cuts

- **WHEN** an exponential Gate pad releases or a group cut or Panic stops it
- **THEN** the Gate uses the selected release curve
- **AND** a group cut or Panic retains its normal stop timing

### Requirement: A pad shapes positive hit velocity

The velocity range SHALL run from zero to 60 dB. A new pad SHALL start with 30 dB. In the decibel curve, velocity 1 SHALL give unity minus the range and 127 SHALL give unity. Intermediate velocities SHALL vary linearly in decibels. Zero velocity remains silent. The Velocity control SHALL offer the Linear legacy value before zero dB. Raw A1 velocities SHALL remain available.

#### Scenario: The velocity range changes both manual and replayed hits

- **WHEN** the owner plays and replays a range of hit velocities
- **THEN** both paths use the pad range and preserve the raw pattern velocities
- **AND** zero range gives the same gain for every positive velocity

## MODIFIED Requirements

### Requirement: The pad panel in blocks

The system SHALL group the parameter rows in this order: TONE, ENV, FILTER, SOURCE, FX, SOUND.
Each block SHALL show its page name above its rows. The current page name SHALL be bright. `Tab` turns the pages in this order, `I` `K` walk
the parameters in this order, and the frame title names the pages in this order.

#### Scenario: Blocks
- **WHEN** the pad edit is open
- **THEN** the panel shows the lines TONE, ENV, FILTER, SOURCE, FX, SOUND in this order, each over its rows

#### Scenario: SOURCE last
- **WHEN** the edit is on the reso of the FILTER page and the owner presses `K`
- **THEN** the program selects Source on SOURCE.
- **AND** `K` moves from the last row of SOURCE to Drive on FX.
- **AND** `K` moves from the last row of FX to Pan on SOUND.
- **AND** `K` moves from the last row of SOUND to Volume on TONE.

### Requirement: Pad edit page cells

The pad edit panel SHALL have six page cells: `tone page`, `env page`, `filter page`, `source page` `fx page` and `sound page` on device pads 9 to 14. A press SHALL open that page in one press. The cell of the current page SHALL be brighter than the other page cells. SMC-PAD LEFT and RIGHT without PAD BANK SHALL go around the pages as Tab and Shift+Tab do. LEFT on TONE SHALL open SOUND. RIGHT on SOUND SHALL open TONE.

#### Scenario: The fx page cell opens FX in one press
- **WHEN** pad edit is open on TONE and the owner presses PAD BANK with device pad 13
- **THEN** the page is FX and the fx page cell is brighter than the other page cells

#### Scenario: LEFT on TONE opens FX
- **WHEN** pad edit is open on TONE, the panel is hidden and the owner presses LEFT
- **THEN** the page is SOUND
