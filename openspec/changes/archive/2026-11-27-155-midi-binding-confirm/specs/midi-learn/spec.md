## MODIFIED Requirements

### Requirement: Pads and banks in a row
The MIDI tab SHALL show the count for pads 1–16 and banks 1–8. A hit SHALL select one control. Enter SHALL save it and advance. A control already saved in the run SHALL stay out of the next step. Esc or P1 SHALL stop the run and keep the saved rows. Backspace SHALL ask before it clears the rows.

#### Scenario: Sixteen pads in a row
- **WHEN** the owner selects "pads 1–16", presses Enter and hits 16 controller pads and presses Enter after each
- **THEN** controller pad k plays pad k, and the row says "16 of 16 learned"

#### Scenario: A double hit
- **WHEN** during the run the owner saves a control and hits the same control again
- **THEN** the run ignores the second hit and the run waits for another control

#### Scenario: Stop in the middle
- **WHEN** the owner saves five pads, hits a sixth control and presses Esc
- **THEN** pads 1–5 stay learned and the row says "5 of 16 learned"

### Requirement: Learn tap and hold on one button

On a button action row, F12 MIDI SHALL let the owner choose tap or hold with J and L before moving a control. Tap SHALL be the default. The pending line SHALL name the selected gesture, the control, and any action that would lose the selected slot. Enter SHALL save only that slot. Esc SHALL keep both existing slots. The other slot on the same control SHALL remain. Pad and bank runs SHALL also wait for Enter.

#### Scenario: Learn two actions on one button
- **WHEN** the owner learns `transport.toggle` as tap and `metronome.toggle` as hold on the same button in GLOBAL
- **THEN** a short press changes transport on release and a hold changes the metronome once
- **AND** both assignments remain after a restart

#### Scenario: Cancel a pending hold
- **WHEN** a button already has a tap, the owner selects a hold action, moves that button, and presses Esc
- **THEN** neither slot changes and the config file stays unchanged

#### Scenario: Reassign one gesture slot
- **WHEN** a button has a tap and a hold, and the owner learns a different hold action on that button
- **THEN** the new action replaces only its hold action
- **AND** the pending warning names the old hold action and the tap remains

#### Scenario: Clear one gesture slot
- **WHEN** a button has a tap and a hold, and the owner clears the row for its hold action
- **THEN** its tap remains and acts on press again
- **AND** no hold action remains after a restart

### Requirement: Binding changes are keyboard only

Binding changes SHALL start and save only from the keyboard. They are F10 Learn, learn and forget on the F12 MIDI tab, reset to profile and the SMC-PAD layout update. The controller SHALL select a control or cancel with P1. The F12 MIDI tab SHALL say «Bindings: keyboard only (F10, Enter)». Other settings SHALL keep their controller routes.

#### Scenario: The MIDI tab names the keyboard path
- **WHEN** the owner opens the F12 MIDI tab at 80×26 and at 120×40
- **THEN** the tab says «Bindings: keyboard only (F10, Enter)»

#### Scenario: The controller can cancel a binding but cannot save it

- **WHEN** a pending binding, a reset, a layout update or a forget question is open
- **THEN** P4 cannot change the map and the screen shows the key to use
- **AND** P1 cancels the pending change
- **AND** Enter saves a new binding when the owner confirms it

### Requirement: Learned confirmation uses the current input owner

While a picker or a question that accepts MIDI confirmation is open, a learned confirm press SHALL reach it before a screen or edit binding. A release, repeated press, or disconnect SHALL NOT add another confirmation. Target limits and input order SHALL stay. Earlier pad releases SHALL reach their original pads.

#### Scenario: Learned confirm answers every current question
- **WHEN** a learned note or CC button sends raw MIDI for each current question type through the shipped profile and dispatch
- **THEN** each press answers the applicable question once when the target permits it, or leaves a binding question open
- **AND** the button release does not answer again or play a pad

#### Scenario: Learned confirm selects in both pickers
- **WHEN** a learned confirm button sends raw MIDI while the path picker or project picker has a selected item
- **THEN** the picker uses only its current target and existing refusal rules
- **AND** no screen action below the picker runs

#### Scenario: Question confirmation wins over a screen binding
- **WHEN** the same controller button has GLOBAL `ui.confirm` and a screen or edit action, then a question opens
- **THEN** its press answers the question without running the screen or edit action
- **AND** outside the question the existing screen or edit action keeps its priority

#### Scenario: Confirm cannot consume an earlier pad release
- **WHEN** a Gate pad starts before a question, a learned confirm button answers it, and the original pad control releases
- **THEN** the original voice receives its release and no new pad or second confirmation starts
