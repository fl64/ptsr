## ADDED Requirements

### Requirement: A1 shows the next live recording step

The A1 overview SHALL use the label space beside the step numbers for one short instruction. An empty pattern SHALL name the effective live record control. A count-in SHALL name its cancel control. During a pass, the instruction SHALL tell the owner to hit pads and name the finish control.

#### Scenario: A1 guides a first recording from keyboard and SMC-PAD

- **WHEN** the owner opens an empty A1, starts a live pass, hits a pad, and finishes through either input
- **THEN** A1 names the effective controls to start, cancel the count-in, and finish the pass
- **AND** the instruction tells the owner to hit pads during recording

### Requirement: A1 playback cue follows accepted state

The A1 instruction SHALL ask the owner to wait during a pending start or finish. With hits present, it SHALL name the effective play or stop control for the accepted playback state. It SHALL name an audio wait or failure when audio cannot accept a pass.

#### Scenario: A1 offers playback only for the accepted state

- **WHEN** the owner finishes a pass, waits for acceptance, pauses or stops the loop, and starts it again
- **THEN** the instruction says to wait during publication, then follows the actual playback state

#### Scenario: A1 omits unavailable actions

- **WHEN** audio is unavailable, a keyboard binding is absent, or another mode takes priority
- **THEN** the instruction shows the reason, names only effective controls, or gives the space to the current mode

### Requirement: A1 first step fits both terminal sizes

The instruction SHALL fit at 80x26 and 120x40. It SHALL keep the step numbers, bank index, help, and safety messages readable.

#### Scenario: A1 next step preserves the screen frame

- **WHEN** the overview draws at each supported size with an empty pattern, a pass in progress, or a recorded loop
- **THEN** the instruction fits beside the step numbers without changing the bank rows or covering warnings
