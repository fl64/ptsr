# master-volume Specification

## Purpose
Master volume of the whole output: keys, a learnable knob with soft takeover, status line, session.

## Requirements

### Requirement: Master volume
The system SHALL let the owner change the volume of the whole output from −40 to +6 dB in 1 dB steps with keys on every screen and with a learned knob, change it smoothly without clicks, show it in the status line and keep it in the session.

#### Scenario: Quieter while playing
- **WHEN** a pad sounds and the owner presses `[` three times
- **THEN** the sound gets 3 dB quieter without a click and the status shows "master -3 dB"

#### Scenario: Knob with soft takeover
- **WHEN** the master volume is −10 dB and the owner turns a learned knob that stands at
  its maximum
- **THEN** the master volume goes to +6 dB at once and glides there without a click

#### Scenario: Restart
- **WHEN** the owner sets the master to −6 dB and restarts the program
- **THEN** the master volume is −6 dB

### Requirement: The meter names limiter activity

The output meter SHALL show a limiter mark when the master reduces gain by more than 1 dB. The mark SHALL fit at both supported terminal sizes and SHALL preserve higher-priority warnings.

#### Scenario: Limiter feedback names a loud master

- **WHEN** the master reduces gain by more than 1 dB during playback
- **THEN** the meter shows the limiter mark at 80 by 26 and 120 by 40
- **AND** a warning or question stays visible
