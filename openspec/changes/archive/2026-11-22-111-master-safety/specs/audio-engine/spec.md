## ADDED Requirements

### Requirement: The master limits excessive peaks

The master SHALL use a linked stereo limiter. It SHALL keep both output samples at or below minus 0.3 dBFS. Use 1.5 ms of look-ahead and a 75 ms release. The master SHALL keep unity gain below minus 6 dBFS. At 0 dB master gain, a gain of one defines the reference.

#### Scenario: The loud master stays below the sample ceiling

- **WHEN** a loud mix uses 32 voices and both shared FX slots
- **THEN** each master sample stays at or below minus 0.3 dBFS
- **AND** the 4x reference render stays at or below 0 dBTP
- **AND** the device mix stays within full scale with the click enabled

#### Scenario: A normal master keeps its level

- **WHEN** a normal mix peaks below minus 6 dBFS
- **THEN** the output has the same level after alignment for the fixed delay
- **AND** the limiter gives no gain reduction indication

### Requirement: The limiter uses fixed device latency

The device output SHALL add 1.5 ms of latency rounded to the nearest whole frame. The click and mastered signal SHALL have the same delay. The delay SHALL NOT move recorded ticks, pattern clock positions or quantized boundaries. The audio callback SHALL retain the five-millisecond Panic deadline at the device.

#### Scenario: Panic meets its deadline through the limiter

- **WHEN** the owner uses Panic during loud playback with a click and effect tails
- **THEN** the old sound is silent at the device within five milliseconds
- **AND** later hits play and the old tail does not return

### Requirement: Shared FX returns have independent gain and mute

Each shared FX slot SHALL have a return from 0 to 100 percent and a mute. Both controls SHALL smooth gain changes. Changing one return SHALL keep the dry signal and the other return at their existing levels.

#### Scenario: A return changes only the selected effect

- **WHEN** two shared effects sound and the owner changes or mutes one return
- **THEN** only that wet path changes at the output
- **AND** it changes with no discontinuity
