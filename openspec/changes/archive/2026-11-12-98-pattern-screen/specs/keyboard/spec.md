# keyboard Delta

## ADDED Requirements

### Requirement: Pattern view keys

Pads SHALL bind Alt+Up to `pattern.view_next` and Alt+Down to `pattern.view_prev`. Alt+Up SHALL select a finer view and Alt+Down a coarser view, and both SHALL stop at the ends. The keys SHALL change only the view. Pad parameters, raw rows, history, dirty state and sound routing SHALL stay unchanged. The keys SHALL work in both supported layouts and input protocols.

#### Scenario: Pattern view keys change only the view
- **WHEN** the owner presses Alt+Up and Alt+Down on Pads in extended and legacy input, then plays a pad in either layout
- **THEN** the view becomes finer or coarser and stops at 1/64 and 1/4
- **AND** raw rows, history and dirty state stay unchanged, and the pad press still plays its pad

### Requirement: Pattern view keys preserve owner input rules

New view defaults SHALL keep explicit owner bindings, `none`, active edit-layer precedence and existing shadow warnings. Text, pickers, questions, F1 and learning SHALL keep their input priority. Outside Pads, a view command SHALL explain that it needs Pads. Pad-block keys SHALL remain pads.

#### Scenario: Pattern view keys preserve owner input rules
- **WHEN** owner overrides hide or move a view key, the edit layer or help is open, or a view command arrives on another screen
- **THEN** effective bindings keep priority, a hidden default gives a startup warning, and blocked commands change nothing
- **AND** a view command outside Pads gives an instruction to open Pads
