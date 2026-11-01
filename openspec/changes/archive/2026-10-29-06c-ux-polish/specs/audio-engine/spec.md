## MODIFIED Requirements

### Requirement: Low-latency sound output
The system SHALL output sound through JACK (PipeWire-JACK) with the buffer size set in the config (128 samples by default), and show the latency in the status line.

#### Scenario: Start with PipeWire running
- **WHEN** the owner starts the program on a system with PipeWire-JACK
- **THEN** the program connects to the sound server and shows in the status the buffer size, the sample rate and the latency in milliseconds

#### Scenario: Sound server unavailable
- **WHEN** JACK/PipeWire is unavailable
- **THEN** the program does not crash and tries the fallback output; when nothing works, the corner indicator shows "no audio", the status line gets one short plain message (for example "No sound: the audio server is not available"), and the internal error text goes to the log only — the status line stays free for other messages
