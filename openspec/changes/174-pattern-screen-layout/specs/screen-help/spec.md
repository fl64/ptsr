## MODIFIED Requirements

### Requirement: A1 overview help explains its live controls and marks

The Pattern screen help band SHALL show the effective F2, bank, view, all-rows and setup controls.
F1 SHALL explain the three velocity glyphs, `~`, the count digits and the velocity lane.
F1 SHALL also explain the detail line, the setup line, the playback plate and the row list.
An absent pattern SHALL give the effective recording action. Owner overrides and `none` SHALL change the shown keys.
Existing notice priority SHALL keep save and audio warnings visible.

#### Scenario: Overview help follows effective bindings
- **WHEN** the owner opens the Pattern screen with default keys, moved keys, or explicit `none`, then opens F1
- **THEN** the help band and F1 name only effective actions and explain event marks, velocity, the lines and row scope
- **AND** the help returns to the same Pattern screen without changing the pattern or hiding a safety warning

### Requirement: F5 names the pattern destination

Short help on other screens SHALL name the F5 destination `pattern`. The Pattern screen title SHALL be `F5 PATTERN`. The title and short labels SHALL NOT use `A1` for this screen.

#### Scenario: Pattern key names its destination
- **WHEN** the owner reads short help on Files and opens the Pattern screen with F5
- **THEN** short help names `pattern` and the title shows `F5 PATTERN` without `A1`

### Requirement: A1 shows the next live recording step

The Pattern screen SHALL use the detail line for one short instruction until the owner selects a cell, and during a pass. An empty pattern SHALL name the effective live record control. A count-in SHALL name the effective Back key or SMC-PAD command P1. If Hold panel is on and the SMC-PAD supports it, the cue SHALL include PAD BANK. If the Back key has no binding, the keyboard cue SHALL name the record control. During a pass, the instruction SHALL tell the owner to hit pads and name the finish control.

#### Scenario: A1 guides a first recording from keyboard and SMC-PAD

- **WHEN** the owner opens an empty pattern, starts a live pass, hits a pad, and finishes through either input
- **THEN** the Pattern screen names the effective controls to start, cancel the count-in, and finish the pass
- **AND** the instruction tells the owner to hit pads during recording

### Requirement: A1 first step fits both terminal sizes

The instruction SHALL fit at 80x26 and 120x40. It SHALL keep the header line, the beat numbers, help, and safety messages readable.

#### Scenario: A1 next step preserves the screen frame

- **WHEN** the Pattern screen draws at each supported size with an empty pattern, a pass in progress, or a recorded loop
- **THEN** the instruction fits in its line without changing the pad rows or covering warnings

### Requirement: A1 tap tempo keeps the bank row visible

On the Pattern screen, the notice row SHALL show the first keyboard tap and the BPM after an accepted second tap. The first cue SHALL expire after two seconds or on the second accepted tap. It SHALL take the place of an earlier tempo result. The header line and the pad rows SHALL remain visible. Warnings and errors SHALL keep priority.

#### Scenario: A1 tap tempo uses the notice row
- **WHEN** the owner taps tempo twice on the Pattern screen at 80x26 or 120x40
- **THEN** a notice names the new tempo and the header line and pad rows remain complete
- **AND** a higher-priority warning keeps the notice row

#### Scenario: A1 shows the first keyboard tempo tap without a controller
- **WHEN** the owner presses Alt+B once on the Pattern screen without an SMC-PAD at 80x26 or 120x40
- **THEN** the notice row says «TAP · 1 hit · tap again» and the tempo stays the same
- **AND** the header line and pad rows remain complete
- **AND** the cue disappears after its tap window or on an accepted second tap

### Requirement: A1 names the accepted transport and recording state

The Pattern screen header line SHALL show Paused with its accepted position or Stopped at the start. With hits present, its cue SHALL name resume for a control that continues, or restart for a control that returns to the start. During recording, the header line SHALL show First pass if the pattern began with no hits or Overdub if it began with hits. That label SHALL stay for the pass. The step position, grid, and recording quantize SHALL remain visible in the detail and setup lines.

#### Scenario: A1 distinguishes pause and reset
- **WHEN** the owner pauses the pattern and then presses STOP again through keyboard or raw MIDI
- **THEN** the Pattern screen first shows the paused position and the resume cue, then shows Stopped and the play cue

#### Scenario: A1 labels the pass from its initial hits
- **WHEN** the owner starts a pass with an empty pattern or a pattern with hits
- **THEN** the Pattern screen shows First pass or Overdub for the duration of that pass

### Requirement: Narrow A1 keeps its navigation help with MIDI

At 80x26 with an SMC-PAD, the Pattern screen footer SHALL use one status row and two full-width short help rows. The header line SHALL keep the bar page, and the detail line SHALL keep the step position. Short help SHALL name the bank, view and full Help controls. In Step Edit it SHALL name the pad action and the exit control.

#### Scenario: Compact A1 keeps navigation help with MIDI
- **WHEN** the owner opens the Pattern screen or enters Step Edit at 80x26 with the SMC-PAD connected
- **THEN** short help shows the navigation or edit actions for that mode

### Requirement: A1 Setup shows the controls that apply

The Pattern screen SHALL show the Setup route when the pattern is absent and Step Edit when the pattern exists with no hits. The Setup footer SHALL show selection, value and exit controls, also with Help open. At 80x26, Setup SHALL use the work area below the header line. On the Pattern screen and in Setup, Full Help SHALL cover the command panel at both sizes. Full Help SHALL show Setup keys first and omit blocked edit actions. When Enter is bound, the current row SHALL determine its action.

#### Scenario: Absent A1 offers Setup
- **WHEN** the owner opens the Pattern screen before the pattern exists
- **THEN** the screen says how to record and names the Setup route
- **AND** if the pattern exists with no hits, Step Edit remains available

#### Scenario: A1 Setup shows only applicable edit actions
- **WHEN** the owner opens Setup and reads the footer or F1
- **THEN** the instructions show the Setup controls and omit blocked pad editing actions
- **AND** the Setup controls appear at the start of Full Help and stay in the footer
- **AND** the command panel leaves Full Help readable when the panel is active
