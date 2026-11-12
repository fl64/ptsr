## MODIFIED Requirements

### Requirement: Pattern view keys

Pads and the A1 overview SHALL bind Alt+Up to `pattern.view_next` and Alt+Down to `pattern.view_prev`. Alt+Up SHALL select a finer view and Alt+Down a coarser view, and both SHALL stop at the ends. The keys SHALL change only the view. Pad parameters, raw rows, history, dirty state and sound routing SHALL stay unchanged. The keys SHALL work in both supported layouts and input protocols.

#### Scenario: Pattern view keys change only the view
- **WHEN** the owner presses Alt+Up and Alt+Down on Pads in extended and legacy input, then plays a pad in either layout
- **THEN** the view becomes finer or coarser and stops at 1/64 and 1/4
- **AND** raw rows, history and dirty state stay unchanged, and the pad press still plays its pad

#### Scenario: Overview view keys share the Pads division
- **WHEN** the owner changes the view on Pads, opens the overview and changes it again
- **THEN** both screens show the same division and the view stops at 1/64 and 1/4
- **AND** neither change edits the pattern or pad parameters

### Requirement: Pattern view keys preserve owner input rules

New view defaults SHALL keep explicit owner bindings, `none`, active edit-layer precedence and existing shadow warnings. Text, pickers, questions, F1 and learning SHALL keep their input priority. Outside Pads and the A1 overview, a view command SHALL explain that it needs one of those screens. Pad-block keys SHALL remain pads.

#### Scenario: Pattern view keys preserve owner input rules
- **WHEN** owner overrides hide or move a view key, the edit layer or help is open, or a view command arrives on another screen
- **THEN** effective bindings keep priority, a hidden default gives a startup warning, and blocked commands change nothing
- **AND** a view command outside Pads and the A1 overview gives an instruction to open one of them

## ADDED Requirements

### Requirement: A1 overview has a direct screen key

F5 SHALL open the A1 overview from an ordinary screen. F2 and unclaimed Esc SHALL return to Pads. Other screen keys SHALL keep their existing screen roles. The overview SHALL work with absent, empty, stopped and playing A1. Opening it SHALL NOT create a pattern or move transport. Pad keys SHALL play and select pads as on Pads. Shift on a pad SHALL give full velocity. Releases SHALL keep their original sound owner across screen and bank changes.

#### Scenario: Overview opens without editing or stopping A1
- **WHEN** the owner opens the overview with F5 while A1 is absent, empty, stopped or playing
- **THEN** it shows the correct state without creating a pattern, restarting transport or changing recorded data
- **AND** F2 or Esc returns to Pads with the same bank and view division

#### Scenario: Overview pad input keeps sound ownership
- **WHEN** a keyboard or raw MIDI pad starts a Gate in the overview and the owner changes bank or screen before release
- **THEN** the pad plays with its original velocity and its release stops the original note
- **AND** no new-bank sound or pattern edit follows the old release

### Requirement: Overview can show empty banks from the keyboard

Alt+H SHALL switch the visibility of empty bank sections while the overview is open. The key SHALL have no effect on pad mode or A1. Other screens SHALL keep their existing Alt+H behavior. Owner overrides and `none` SHALL keep their existing priority rules.

#### Scenario: Overview empty-bank key keeps pad and pattern state
- **WHEN** the owner presses Alt+H in the overview with empty banks hidden, then presses it again
- **THEN** empty sections appear and disappear without changing the current bank, pad modes or A1
