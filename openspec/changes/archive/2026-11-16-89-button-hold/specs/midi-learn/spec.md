# midi-learn Delta

## MODIFIED Requirements

### Requirement: One control, one action

The system SHALL keep each learned control on one immediate action per group. A button with a hold binding MAY have one tap action and one hold action in the same group. Assigning a new action to a gesture slot SHALL remove the old action from that slot only. The control SHALL keep its actions in the other groups. On a screen, the screen group SHALL take priority over the global group.

#### Scenario: Reassign a button
- **WHEN** the owner learns a button for "bank next" in GLOBAL and for "pad mode" in PADS
- **THEN** the button switches the pad mode on the pads screen
- **AND** it changes the bank on other screens
- **AND** learning shows `bank next · stays` before the owner presses Enter

## ADDED Requirements

### Requirement: Learn tap and hold on one button

On a button action row, F12 MIDI SHALL let the owner choose tap or hold with J and L before moving a control. Tap SHALL be the default. The pending line SHALL name the selected gesture, the control, and any action that would lose the selected slot. Enter SHALL save only that slot. Esc SHALL keep both existing slots. The other slot on the same control SHALL remain. Pad and bank learning runs SHALL keep their immediate press behavior.

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

### Requirement: Saved button actions keep their old timing

The user config SHALL accept an existing action string as an immediate press binding. A new tap and hold binding SHALL use one table on the control with `tap` and `hold` keys. The new binding format SHALL NOT by itself rewrite an old string. The existing FX binding migration MAY save the user config. Adding a hold to that control SHALL preserve its old action as tap, which then acts on release.

#### Scenario: Old saved action stays immediate
- **WHEN** the program loads an existing user config that binds a button to one action string
- **THEN** that action runs on press, the MIDI row shows the binding, and a save for another reason keeps the binding as a string

#### Scenario: Add a hold to an old binding
- **WHEN** the owner learns a hold on a button whose saved string action already runs on press
- **THEN** the old action becomes its tap and runs on release
- **AND** the new hold runs at the threshold or when the owner uses another control

### Requirement: Learned button slots stay visible and valid

The MIDI list and profile seeding SHALL show and save both gesture slots. The program SHALL remove an invalid action name with the existing warning. It SHALL keep a valid action in the other slot.

#### Scenario: Seed a profile with two actions
- **WHEN** a new device profile gives one button tap and hold actions and the owner connects the device for the first time
- **THEN** the program copies both actions to the user config once
- **AND** later owner edits to either slot win on restart
