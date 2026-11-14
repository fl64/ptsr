# screen-help Delta

## MODIFIED Requirements

### Requirement: A1 overview help explains its live controls and marks

The overview help band SHALL show effective F2, bank, view, and empty-row controls. F1 SHALL explain event symbols, velocity levels, the playback plate, bank sections, and the filtered or full table. An absent A1 SHALL give the effective recording action. Owner overrides and `none` SHALL change the shown keys. Existing notice priority SHALL keep save and audio warnings visible.

#### Scenario: Overview help follows effective bindings
- **WHEN** the owner opens the overview with default keys, moved keys, or explicit `none`, then opens F1
- **THEN** the help band and F1 name only effective actions and explain event marks, velocity, and row scope
- **AND** the help returns to the same overview without changing A1 or hiding a safety warning

### Requirement: A1 correction help names its active controls

The overview help SHALL show the effective F7 entry key. In correction it SHALL state that J/L or learned MIDI step controls choose a time step. It SHALL state that a pad press adds or removes that pad's hit at this step. It SHALL name event choice, view, bank, Undo, and exit controls. F1 SHALL explain the time column, selected pad row, audio plate, multiple-event choice, and silent insertion. Owner overrides and `none` SHALL change the shown keys.

#### Scenario: Correction help follows the next action
- **WHEN** the owner opens the overview, enters correction, selects empty and multiple-event cells, then opens F1
- **THEN** help names the applicable addition, selection, removal, or exit action with effective keys
- **AND** it does not describe an edit pad press as live playing

#### Scenario: Correction help separates step from pad target
- **WHEN** the owner moves the time cursor and presses a different pad at that step
- **THEN** help and F1 identify keyboard or learned MIDI step controls as time selection and pad keys as row selection and hit toggle
- **AND** the header and result use the same step number

### Requirement: A1 correction feedback states one-hit scope

Only a successful addition, removal, or Undo SHALL report a changed event. The result SHALL name its bank, pad, step, and one-hit scope and direct the owner to Undo. The help band SHALL show the effective Undo key, including `none`. A refusal SHALL name absent A1, a conflicting owner, full capacity, or stale event as applicable and give a safe next action. Save and audio warnings SHALL keep their existing priority.

#### Scenario: Correction feedback separates focus from edits
- **WHEN** an empty-cell addition, multiple-event focus, removal, Undo, and stale selection occur in order
- **THEN** only addition, removal, and Undo claim a changed hit, and each names one-event scope
- **AND** a refusal or F1 preserves the edit target and existing safety warnings

#### Scenario: Correction feedback separates focus from removal
- **WHEN** a multiple-event focus, removal, Undo, and stale selection occur in order
- **THEN** only removal and Undo claim a changed hit, and each names one-event scope
- **AND** a refusal or F1 preserves the edit target and existing safety warnings
