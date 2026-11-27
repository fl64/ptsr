## ADDED Requirements

### Requirement: A1 shows the selected position and recording settings

The A1 overview and Step Edit SHALL show the selected bar and step with their totals.
The screen SHALL show the view grid and recording quantize separately.
At 80x26 and 120x40, both lines SHALL fit.
The selected time column SHALL have a mark distinct from the playback mark.
The keyboard step actions and SMC-PAD STEP SHALL move the selected column.

#### Scenario: A1 position remains visible across a bar line
- **WHEN** the owner uses a keyboard step action or turns SMC-PAD STEP across a bar line
- **THEN** the marked time column and total step position move to the next cell
- **AND** the total bars, view grid, and recording quantize remain readable at both supported sizes

#### Scenario: A1 selection stays visible while playback passes another bar
- **WHEN** the owner selects a time cell and playback moves beyond that page
- **THEN** the selected column and visible page remain at the chosen time
- **AND** the screen shows the playback position separately

### Requirement: A1 keeps the selected position during correction

Entry to Step Edit SHALL use the overview position when the owner selects a cell.
F7, Esc, and command pad 1 SHALL return to the overview with the last edit position.
The existing input owners SHALL retain their priority.
A pad edit result SHALL show the target pad, the global step, and the action.

#### Scenario: A1 keeps the chosen step through edit and Undo
- **WHEN** the owner selects a later step, enters Step Edit, adds a hit, then uses Undo and Back
- **THEN** the pad edit reports its pad, step, and result
- **AND** Undo restores the hit state and the overview keeps the edited step

## MODIFIED Requirements

### Requirement: A1 step correction needs an explicit mode

The owner SHALL enter correction only from the A1 overview with an existing A1.
Entry and exit SHALL leave transport, raw hits, dirty state and history unchanged.
A conflicting input owner or pending capture SHALL refuse entry without ending its work.

#### Scenario: Step correction entry keeps A1 playing
- **WHEN** the owner requests correction with absent, empty, stopped and playing A1
- **THEN** absent A1 gives the effective record action, while existing A1 enters without moving transport
- **AND** entry uses the chosen cell, or the playback or first cell before the owner chooses a position

#### Scenario: Step correction refuses conflicting work
- **WHEN** capture, recovery, placement, marking, pad edit, pattern controls, audio recording or a question owns input
- **THEN** correction refuses with a next action and does not change the existing work
- **AND** text, pickers, learning and original pad releases keep their priority
