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

The pad SHALL have linear or exponential decay and Gate release. Attack SHALL remain linear. Hold SHALL run from zero to 2000 ms between attack and decay. The exponential stages SHALL reach minus 60 dB at their set duration and fade to zero. Panic, a voice steal and a mute group SHALL keep their timing.

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
