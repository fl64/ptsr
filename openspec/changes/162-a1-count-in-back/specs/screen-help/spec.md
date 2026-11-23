## MODIFIED Requirements

### Requirement: A1 shows the next live recording step

The A1 overview SHALL use the label space beside the step numbers for one short instruction. An empty pattern SHALL name the effective live record control. A count-in SHALL name the effective Back key or SMC-PAD command P1. If Hold panel is on and the SMC-PAD supports it, the cue SHALL include PAD BANK. If the Back key has no binding, the keyboard cue SHALL name the record control. During a pass, the instruction SHALL tell the owner to hit pads and name the finish control.

#### Scenario: A1 guides a first recording from keyboard and SMC-PAD

- **WHEN** the owner opens an empty A1, starts a live pass, hits a pad, and finishes through either input
- **THEN** A1 names the effective controls to start, cancel the count-in, and finish the pass
- **AND** the instruction tells the owner to hit pads during recording
