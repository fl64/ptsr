## ADDED Requirements

### Requirement: A panel cell hit lights the cell

A pad hit on a visible panel cell SHALL light that cell for 120 ms, the time of a pad hit flash.
An active cell SHALL light in its own color. A dim or empty cell SHALL light grey and run no action.
A hit that the panel does not take SHALL light no cell.

#### Scenario: A cell hit lights the cell
- **WHEN** the owner hits an active cell and then a dim cell on the visible panel
- **THEN** each hit lights its own cell for 120 ms, the active one in its color and the dim one grey
- **AND** the dim cell runs no action
