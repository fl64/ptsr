## ADDED Requirements

### Requirement: A1 tap tempo keeps the bank row visible

On A1, the applied result of a tap tempo sequence SHALL use the notice row. The notice SHALL show BPM. The bank and step rows SHALL remain visible. A higher-priority warning SHALL keep priority.

#### Scenario: A1 tap tempo uses the notice row
- **WHEN** the owner taps tempo twice on A1 at 80x26 or 120x40
- **THEN** a notice names the new tempo and the bank and step rows remain complete
- **AND** a higher-priority warning keeps the notice row

### Requirement: A1 names the accepted transport and recording state

The A1 title SHALL show Paused with its accepted position or Stopped at the start. With hits present, its cue SHALL name resume for a control that continues, or restart for a control that returns to the start. During recording, the title SHALL show First pass if A1 began with no hits or Overdub if it began with hits. That label SHALL stay for the pass. The step position, grid, and recording quantize SHALL remain visible.

#### Scenario: A1 distinguishes pause and reset
- **WHEN** the owner pauses A1 and then presses STOP again through keyboard or raw MIDI
- **THEN** the overview first shows the paused position and the resume cue, then shows Stopped and the play cue

#### Scenario: A1 labels the pass from its initial hits
- **WHEN** the owner starts a pass with empty A1 or an A1 with hits
- **THEN** the overview shows First pass or Overdub for the duration of that pass
