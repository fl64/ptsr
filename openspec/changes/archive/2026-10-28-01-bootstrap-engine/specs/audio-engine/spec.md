## ADDED Requirements

### Requirement: Low-latency sound output
The system SHALL output sound through JACK (PipeWire-JACK) with the buffer size set in the config (128 samples by default), and show the latency in the status line.

#### Scenario: Start with PipeWire running
- **WHEN** the owner starts the program on a system with PipeWire-JACK
- **THEN** the program connects to the sound server and shows in the status the buffer size, the sample rate and the latency in milliseconds

#### Scenario: Sound server unavailable
- **WHEN** JACK/PipeWire is unavailable
- **THEN** the program does not crash, shows a clear message in English and tries the fallback output

### Requirement: Audio thread does not block
The system SHALL process sound in the audio thread without memory allocation, locks or I/O; communication with other threads goes only through lock-free queues and atomics.

#### Scenario: UI hangs
- **WHEN** the UI thread does not respond for 2 seconds
- **THEN** the sound keeps playing without dropouts

### Requirement: Sound observability
The system SHALL show in the status line the audio thread load in percent and a counter of buffer underruns (xruns).

#### Scenario: Buffer underrun
- **WHEN** the audio thread did not fill the buffer in time
- **THEN** the xrun counter goes up and is highlighted

### Requirement: Offline render
The system SHALL be able to play a scenario of events without a sound card and save the result to WAV, using the same engine as live.

#### Scenario: Render of a test pattern
- **WHEN** a render of a scenario of 8 clicks with a step of 1/8 at 120 BPM is run
- **THEN** a WAV is created where each click starts at its own sample with ±1 precision, without clipping and NaN
