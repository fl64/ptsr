## MODIFIED Requirements

### Requirement: Record the output device

While output recording is active, the system SHALL record the sampler master and the other applications on the selected output before the sampler click. The key and learned control SHALL toggle the same recording. If the device has no monitor or the system cannot link the recording inputs, the system SHALL report the cause and refuse to start. When recording stops, the input links for the take SHALL close.

#### Scenario: Capture what plays

- **WHEN** music plays on the output device and the owner starts and stops output recording
- **THEN** the recording is in hand
- **AND** playback contains the sampler and other applications on the chosen output
- **AND** the recording has no sampler click

#### Scenario: No monitor on the device

- **WHEN** the output device exposes no monitor ports and the owner starts the output recording
- **THEN** the status says the device has no monitor to record and nothing starts

#### Scenario: Follow other playback sources during an output recording

- **WHEN** another application connects to or leaves the selected output during a recording
- **THEN** the system updates the recording links and keeps the take
- **AND** links from other outputs stay out of the capture

#### Scenario: Closing the output preserves the captured part

- **WHEN** the selected output disappears while output recording is active
- **THEN** the system stops the take and reports the loss
- **AND** retained audio follows the usual recording handoff
