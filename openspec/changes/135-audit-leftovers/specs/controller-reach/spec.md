## ADDED Requirements

### Requirement: Learn warns before it replaces a panel cell

A pending Learn binding can fall on an SMC-PAD panel cell. Then the prompt SHALL name the cell by its printed number and its current label before the save.

#### Scenario: Learn names the replaced panel cell
- **WHEN** the owner learns pad mode and touches the panel cell of device pad 6 in the screens view
- **THEN** the prompt says that the binding replaces panel cell P6 open
