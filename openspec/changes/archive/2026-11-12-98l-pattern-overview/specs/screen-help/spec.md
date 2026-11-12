## ADDED Requirements

### Requirement: A1 overview help explains its live controls and marks

The overview help band SHALL show effective F2, bank, view and empty-bank controls. F1 SHALL explain all event symbols, the playback plate, bank sections and the read-only table. An absent A1 SHALL give the effective recording action. Owner overrides and `none` SHALL change the shown keys. Existing notice priority SHALL keep save and audio warnings visible.

#### Scenario: Overview help follows effective bindings
- **WHEN** the owner opens the overview with default keys, moved keys or explicit `none`, then opens F1
- **THEN** the help band and F1 name only effective actions and explain the event marks and bank scope
- **AND** the help returns to the same overview without changing A1 or hiding a safety warning

### Requirement: F5 names the pattern destination

Short help on other screens SHALL name the F5 destination `pattern`. The overview title SHALL name `PATTERN` and `A1`. A short label SHALL NOT use `A1` alone for this screen action.

#### Scenario: Pattern key names its destination
- **WHEN** the owner reads short help on Files and opens the overview with F5
- **THEN** short help names `pattern` and the overview title names `PATTERN` and `A1`
