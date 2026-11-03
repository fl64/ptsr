## ADDED Requirements

### Requirement: Learn the keymap cleanup actions

The system SHALL list `pad.copy` in PADS and `chop.assign_order` and `chop.hits_back`
in CHOP on the F12 MIDI tab. F10 SHALL select these commands by Ctrl+C on Pads, Tab
and Shift+H on Chop, or their effective owner keys. Learning SHALL NOT execute them.
They SHALL accept a controller button or pad, not an absolute knob. A learned copy
command SHALL take the selected pad in hand; another controller pad or keyboard pad
SHALL be a target. Assign-order and hits-back SHALL follow their Chop mode guards.
Their saved bindings SHALL survive restart. Old `edit.param_next` bindings SHALL keep
switching assignment order. No existing owner binding or X-Session default SHALL be
reset or reassigned by this cleanup.

#### Scenario: Cleanup actions appear in their learn groups
- **WHEN** the owner opens the F12 MIDI tab
- **THEN** PADS lists `pad.copy` and CHOP lists `chop.assign_order` and `chop.hits_back`

#### Scenario: Pick pad copy in screen learn
- **WHEN** F10 learn is open on Pads and the owner presses Ctrl+C
- **THEN** `pad.copy` is selected, no copy is taken and no pad sounds

#### Scenario: Pick assign order in screen learn
- **WHEN** F10 learn is open on Chop and the owner presses Tab
- **THEN** `chop.assign_order` is selected and no plan or slice changes

#### Scenario: Pick hits back in screen learn
- **WHEN** F10 learn is open on Chop and the owner presses Shift+H
- **THEN** `chop.hits_back` is selected and no cuts or threshold change

#### Scenario: Learn a button for pad copy
- **WHEN** a button is bound to `pad.copy`, the owner presses it on Pads and hits a target pad
- **THEN** it takes and places the same copy as Ctrl+C and a keyboard target
- **AND** the button release does not take another copy or play a pad

#### Scenario: Learn a button for assign order
- **WHEN** a button bound to `chop.assign_order` is pressed while a plan is open
- **THEN** it switches by-hand and in-order exactly once and does not apply the plan
- **AND** outside assignment it does nothing

#### Scenario: Learn a button for hits back
- **WHEN** a button bound to `chop.hits_back` is pressed in Chop listen mode
- **THEN** it raises the threshold one step with the same cuts and undo rules as Shift+H
- **AND** it stays ignored during assignment

#### Scenario: Cleanup learned bindings survive restart
- **WHEN** the owner saves bindings for all three new actions and restarts
- **THEN** they keep their assigned controls and all previous learned bindings stay unchanged

#### Scenario: Cleanup does not reset X Session controls
- **WHEN** the cleanup is installed with an initialized or owner-modified X-Session profile
- **THEN** all its controls keep their roles without a reset or new unsafe direct command
