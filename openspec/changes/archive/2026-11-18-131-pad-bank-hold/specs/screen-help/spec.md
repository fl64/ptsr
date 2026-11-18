## MODIFIED Requirements

### Requirement: Help explains current SMC-PAD panel roles

With the SMC-PAD connected, F1 SHALL show the current panel view and 16 physical cell labels. It SHALL show PLAY and STOP roles and the current PAD BANK rule: a hold, or the first second-bank hit without the shortcut. It SHALL name unavailable cells as unavailable. It SHALL not describe SHIFT as a ptsr modifier. The keyboard help and X-Session view SHALL keep their paths.

#### Scenario: Help names panel entry and current cells
- **WHEN** the owner opens F1 before and after the panel shows the screens view
- **THEN** F1 explains how PAD BANK shows the panel and lists the visible cells in physical order
- **AND** it names Redo and Panic as unavailable and keeps keyboard commands visible
