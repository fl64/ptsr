## ADDED Requirements

### Requirement: The meter names limiter activity

The output meter SHALL show a limiter mark when the master reduces gain by more than 1 dB. The mark SHALL fit at both supported terminal sizes and SHALL preserve higher-priority warnings.

#### Scenario: Limiter feedback names a loud master

- **WHEN** the master reduces gain by more than 1 dB during playback
- **THEN** the meter shows the limiter mark at 80 by 26 and 120 by 40
- **AND** a warning or question stays visible
