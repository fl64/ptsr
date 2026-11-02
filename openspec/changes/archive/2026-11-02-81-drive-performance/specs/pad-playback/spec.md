## ADDED Requirements

### Requirement: Drive sound stays the same after optimization

The system SHALL preserve the drive transfer curve and the smooth change of drive on a
sounding pad. An offline comparison against the previous formula SHALL differ by at most
0.000001 per sample for fixed drive and for a drive that changes during playback. The
comparison SHALL contain no non-finite samples or clipping. The benchmark SHALL keep
32 voices sounding for its full measurement, including when pitch is above zero.

#### Scenario: The optimized drive matches the previous sound
- **WHEN** the same sample is played with fixed and changing drive using the old formula
  and the optimized engine
- **THEN** the rendered outputs differ by at most 0.000001 per sample and contain no
  non-finite samples or clipping

#### Scenario: All benchmark voices stay sounding
- **WHEN** the 32-voice benchmark runs with pitch above zero
- **THEN** all voices have sample data for the whole measurement
