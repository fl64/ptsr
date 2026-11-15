# midi-learn Delta

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

The user config SHALL accept an existing action string as an immediate press binding. A new tap and hold binding SHALL use one table on the control with `tap` and `hold` keys. Loading an old string SHALL NOT rewrite it. Adding a hold to that control SHALL preserve its old action as tap, which then acts on release. The MIDI list and profile seeding SHALL show and save both slots. Invalid action names SHALL be removed with the existing warning and SHALL NOT remove a valid action in the other slot.

#### Scenario: Old saved action stays immediate
- **WHEN** the program loads an existing user config that binds a button to one action string
- **THEN** that action runs on press, the file is not rewritten, and the MIDI row shows the binding

#### Scenario: Add a hold to an old binding
- **WHEN** the owner learns a hold on a button whose saved string action already runs on press
- **THEN** the old action becomes its tap and runs on release
- **AND** the new hold runs at the threshold or when the owner uses another control

#### Scenario: Seed a profile with two actions
- **WHEN** a new device profile gives one button tap and hold actions and the owner connects the device for the first time
- **THEN** the program copies both actions to the user config once
- **AND** later owner edits to either slot win on restart
