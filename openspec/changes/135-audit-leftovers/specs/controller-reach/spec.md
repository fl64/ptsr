## ADDED Requirements

### Requirement: Learn warns before it replaces a panel cell

A pending Learn binding can fall on an SMC-PAD panel cell. Then the prompt SHALL name the cell by its printed number and its current label before the save.

#### Scenario: Learn names the replaced panel cell
- **WHEN** the PAD BANK hold is off, and the owner learns pad mode on Pads and touches device pad 6 on the PAD BANK layer
- **THEN** the prompt says that the binding replaces panel cell P6 rec out
