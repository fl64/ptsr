# screen-help Delta

## ADDED Requirements

### Requirement: Pattern help names its action

The Pads help group for A1 SHALL say `PATTERN`. The action that opens pattern controls SHALL use the visible text `pattern keys` with its effective binding. With the default binding, it SHALL say `Alt+N: pattern keys`. A missing or disabled binding SHALL NOT show a false key. MIDI learn SHALL show `pattern` for the existing pattern screen action without changing its saved action ID or group.

#### Scenario: Pattern keys follow the effective binding
- **WHEN** the owner opens Pads and F1 with the default, moved, or disabled pattern-controls binding
- **THEN** the help group says `PATTERN` and names only the effective key for `pattern keys`
- **AND** MIDI learn shows `pattern` while old saved assignments still work

## MODIFIED Requirements

### Requirement: The change scope in the help line

The system SHALL show the next edit scope in the Pads help line with the mark toggle. It SHALL say `edit: this pad`, `edit: marked (N)`, or `edit: bank` for the applicable scope. It SHALL name `Ctrl+A` as `mark bank` in F1. The scope SHALL remain visible when a terminal hides Ctrl combinations until the owner holds Ctrl.

#### Scenario: Scope in the help line
- **WHEN** Pads selects one pad, marks three pads, then marks the bank and clears marks
- **THEN** the help line shows `edit: this pad`, `edit: marked (3)`, `edit: bank`, and `edit: this pad` in that order
- **AND** the mark state stays visible and the same pad operations affect the same pads

#### Scenario: The mark keys in the full help
- **WHEN** the owner opens F1 on Pads
- **THEN** it names `Ctrl+A mark bank`, and no key named `scope`
