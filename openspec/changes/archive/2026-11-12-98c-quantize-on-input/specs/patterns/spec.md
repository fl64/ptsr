# patterns Delta

## ADDED Requirements

### Requirement: Quantize applies when a hit is captured

A captured row SHALL get its tick from the Quantize and Strength of A1 at its capture. The 96 rounding rules SHALL apply. Off SHALL keep the captured tick. A Gate duration SHALL stay as captured. A later Quantize or Strength change SHALL NOT move a saved row.

#### Scenario: A saved hit keeps the quantize of its capture
- **WHEN** the owner records a hit with Quantize 1/4, then sets Quantize 1/16 and records another hit
- **THEN** the first row stays on its 1/4 line and the second row lands on the nearest 1/16 line
- **AND** playback plays both rows at their saved ticks

#### Scenario: Strength and Off set the captured tick
- **WHEN** the owner records one off-grid hit with Quantize 1/16 at strengths 0, 50 and 100, and with Off
- **THEN** the saved ticks move 0, 50 and 100 percent toward the line, and Off keeps the captured tick
- **AND** the Gate duration of each row stays as captured

### Requirement: Playback plays saved ticks

Playback SHALL play each visible row at its saved tick. It SHALL NOT quantize the rows again. Same-pad rows on one tick SHALL play once by the 96 collision order.

#### Scenario: A settings change does not move playback
- **WHEN** A1 plays saved rows and the owner changes Quantize and Strength through every value
- **THEN** the rendered onsets stay at the saved ticks within one output frame
- **AND** the rendered output has no clipping, non-finite sample or click

## MODIFIED Requirements

### Requirement: A completed pass replaces same-step hits

When a pass completes, each fresh row SHALL replace older rows of the same absolute pad at the same saved tick. The most recently captured row SHALL stay, also between two fresh rows of one pass. Other ticks, other pads and rows hidden by the loop length SHALL stay. Replacement SHALL NOT occur while the pass records. Pass Undo SHALL restore replaced rows. Load SHALL NOT remove rows.

#### Scenario: A replayed step keeps only the newest hit
- **WHEN** a pass adds a hit on a pad whose older row has the same saved tick, and the pass ends
- **THEN** only the newest row remains at that pad and tick, with its velocity and Gate length
- **AND** later loops play that row once

#### Scenario: Two hits in one cell of one pass keep the last
- **WHEN** the owner hits one pad twice inside one 1/16 cell in one pass with Quantize 1/16
- **THEN** after the pass one row remains at that tick, from the second hit
- **AND** with Quantize 1/32 the two hits land in two cells and both stay

#### Scenario: Replacement keeps other steps, pads and hidden rows
- **WHEN** a completed pass adds hits next to older rows at other ticks, on other pads and beyond a shortened loop
- **THEN** all those older rows remain, and quantize Off keeps hits at different ticks separate
- **AND** a loaded project keeps its saved same-tick rows until a pass replaces them

#### Scenario: Undo restores replaced hits
- **WHEN** the owner undoes a completed pass that replaced older rows
- **THEN** the older rows return with their original order, velocity and Gate length
- **AND** the fresh rows of that pass disappear

### Requirement: Change pattern settings without a step editor

Compact pattern controls SHALL expose bars, quantize and strength on every current screen. Opening without A1 SHALL create an empty default A1. Bars SHALL clamp to 1–64 while recording is Idle. Quantize SHALL use all existing grids and strength steps of 10. Grid and strength changes SHALL apply to later captured rows only. Count-in and Finishing SHALL refuse setting edits.

#### Scenario: Set loop length before recording
- **WHEN** the owner opens pattern controls without A1, changes bars and reaches both length limits
- **THEN** an empty A1 uses the chosen length, values clamp at 1 and 64, without a TOML edit
- **AND** recording phases refuse length changes without changing hidden rows

#### Scenario: Change the recording grid without changing the raw take
- **WHEN** the owner changes among every quantize grid and strength 0, 50 and 100 during recording
- **THEN** later captured rows follow those settings and earlier rows keep their saved ticks
- **AND** CountIn and Finishing refuse those edits with the actual phase as the reason

## REMOVED Requirements

### Requirement: Playback quantize preserves raw timing

**Reason**: The owner chose quantize on capture. A playback quantize moved hits that were already in place.

**Migration**: "Quantize applies when a hit is captured" and "Playback plays saved ticks" replace it. The program does not convert old projects. A saved tick plays as it is.
