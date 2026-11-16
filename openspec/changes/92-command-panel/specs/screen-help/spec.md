# screen-help Delta

## ADDED Requirements

### Requirement: Help explains current SMC-PAD panel roles

With the SMC-PAD connected, F1 SHALL show the current panel page and 16 physical cell labels. It SHALL show PLAY and STOP roles and the first-hit reveal rule. It SHALL name unavailable cells as unavailable. Its help SHALL not describe SHIFT as a ptsr modifier without a measured host message. The ordinary keyboard help and X-Session view SHALL keep their paths.

#### Scenario: Help names panel entry and current cells
- **WHEN** the owner opens F1 before and after the panel reveals its context page
- **THEN** F1 explains that the first second-bank hit only shows a hidden panel and lists the visible page's cells in physical order
- **AND** it names Redo and Panic as unavailable and keeps keyboard commands visible

### Requirement: Panel answers show the active question

A visible question SHALL name the SMC-PAD P7 cancel and P8 confirm paths when the device is connected. The question SHALL retain its current target and keyboard answers. The panel SHALL not advertise PLAY or STOP as confirmation. A controller answer SHALL close the question with its normal success or cancellation notice.

#### Scenario: Question shows both controller answers
- **WHEN** a replacement question opens while the SMC-PAD is connected and the panel is hidden or visible
- **THEN** the question shows P7 cancel and P8 confirm beside its keyboard answers
- **AND** a PLAY or STOP tap leaves the question visible with the same target
