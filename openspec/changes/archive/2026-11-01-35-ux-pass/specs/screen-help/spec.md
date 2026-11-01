## ADDED Requirements

### Requirement: The chop hint follows the work
The system SHALL say in the hint line of the chop screen the next step for the state of the chop: no cuts yet — `Y` or `Space` and `M`; cuts and no slice on a pad — `P` lays them on pads; slices on pads — the bounds move with `Ctrl`/`Alt` and `J` `L` and the pads follow; no hint in the edit.

#### Scenario: The hint after a layout
- **WHEN** the slices of a chop lie on pads
- **THEN** the hint says how to move a bound, not how to start a chop

### Requirement: Combos read as combos without kitty
The system SHALL, on a terminal without the kitty protocol, write every modifier key of the help line as a combination (`Ctrl+Z`, `Ctrl+J L`), the same way for all of them; with the protocol the held-modifier help stays as it is.

#### Scenario: One way to write Ctrl
- **WHEN** the chop screen is open on a terminal without the kitty protocol
- **THEN** undo reads `Ctrl+Z` like `Ctrl+Y`, and the bounds read `Ctrl+J L` and `Alt+J L`
