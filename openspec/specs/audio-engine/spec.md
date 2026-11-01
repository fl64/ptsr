# audio-engine Specification

## Purpose
Audio engine: low-latency sound output without blocking the audio thread, observability (latency, load, xruns) and offline render of a scenario to WAV with the same engine.

## Requirements

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

### Requirement: Offline render
The system SHALL be able to play a scenario of events without a sound card and save the result to WAV, using the same engine as live.

#### Scenario: Render of a test pattern
- **WHEN** a render of a scenario of 8 clicks with a step of 1/8 at 120 BPM is run
- **THEN** a WAV is created where each click starts at its own sample with ±1 precision, without clipping and NaN
