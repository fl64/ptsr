# Spec Delta

## ADDED Requirements

### Requirement: A1 correction help names its active controls

The overview help SHALL show the effective F7 entry key. In correction it SHALL name step, event, view, bank, removal, Undo and exit controls. F1 SHALL explain the edit cursor, audio plate, multiple-event choice and no insertion. Owner overrides and `none` SHALL change the shown keys.

#### Scenario: Correction help follows the next action
- **WHEN** the owner opens the overview, enters correction, selects empty and multiple-event cells, then opens F1
- **THEN** help names the applicable selection, removal or exit action with effective keys
- **AND** it does not describe an edit pad press as live playing or insertion

### Requirement: A1 correction feedback states one-hit scope

Only a successful removal or Undo SHALL report a changed event. The result SHALL name its pad, one-hit scope and effective Undo key. A refusal SHALL name absent A1, a conflicting owner or stale event as applicable and give a safe next action. Save and audio warnings SHALL keep their existing priority.

#### Scenario: Correction feedback separates focus from removal
- **WHEN** an empty press, multiple-event focus, removal, Undo and stale selection occur in order
- **THEN** only removal and Undo claim a changed hit, and each names one-event scope
- **AND** a refusal or F1 preserves the edit target and existing safety warnings
