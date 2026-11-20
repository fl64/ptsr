## ADDED Requirements

### Requirement: Binding changes are keyboard only

Binding changes SHALL start only from the keyboard. They are F10 Learn, learn and forget on the F12 MIDI tab, reset to profile and the SMC-PAD layout update. The controller SHALL give only the control to bind. The F12 MIDI tab SHALL say «Bindings: keyboard only (F10, Enter)». Other settings SHALL keep their controller routes.

#### Scenario: The MIDI tab names the keyboard path
- **WHEN** the owner opens the F12 MIDI tab at 80×26 and at 120×40
- **THEN** the tab says «Bindings: keyboard only (F10, Enter)»

### Requirement: The Setup choose cell changes no binding

The Setup panel SHALL show `choose` on device pad 5. `choose` SHALL act as Enter on rows that change no binding. These rows are a group, the knob response, the hold time, the PAD BANK mode and the Paths folder picker. On an action, set, reset or layout row, `choose` SHALL start nothing and SHALL say «Bindings: keyboard only (F10, Enter)».

#### Scenario: Choose opens a group but starts no learning
- **WHEN** the owner selects a group row on the F12 MIDI tab and presses `choose` from the SMC-PAD, and then selects an action row and presses `choose`
- **THEN** the group opens, and on the action row no learning starts and a notice says «Bindings: keyboard only (F10, Enter)»

## MODIFIED Requirements

### Requirement: Learn a control for an action
The F12 MIDI tab SHALL let the owner select an action and wait for a moved control.
The row SHALL show that control pending without saving.
`Enter` SHALL assign and save it.
`Esc` SHALL drop it without changes.
Another control moved before `Enter` SHALL replace the pending control.
`Backspace` on an assigned row SHALL ask first, name the control and say «This cannot be undone.». `Enter` SHALL remove the binding and `Esc` SHALL keep it.

#### Scenario: Knob for pad volume
- **WHEN** the owner selects "pad volume", presses Enter, turns a knob, touching a neighbour knob briefly, and presses Enter again
- **THEN** the knob that moved most is assigned, the row shows it, and the knob changes the selected pad volume with soft takeover

#### Scenario: Button for the next bank
- **WHEN** the owner selects "bank next", presses Enter, presses a controller button and presses Enter again
- **THEN** that button switches to the next bank; the control the row had before no longer does

#### Scenario: Cancel and clear
- **WHEN** the owner presses Esc while the program waits for a control, or Backspace on an assigned row and then Enter
- **THEN** waiting stops without changes, or the binding is removed and the action has no
  control, also when it came from the profile

#### Scenario: Forget asks and says that it has no undo
- **WHEN** the owner presses Backspace on an assigned row
- **THEN** a question names the control and says «This cannot be undone.», and Esc keeps the binding

#### Scenario: Clear a profile role
- **WHEN** left Cue (b7) opens pad edit from the X-Session Pro profile, and the owner
  clears the "open pad edit only" row and restarts the program
- **THEN** b7 does nothing on the pads screen, and the row still has no control

### Requirement: Screen learn clearing needs confirmation

`Backspace` SHALL make "none" pending and SHALL say «This cannot be undone.».
`Enter` SHALL clear the selected element's control, and `Esc` SHALL keep it.

#### Scenario: Clear waits for Enter
- **WHEN** the cutoff, bound to k5, is picked and the owner presses `Backspace`
- **THEN** the label says `k5 → none` in the pending colour and the block says «This cannot be undone.»; after `Enter` the cutoff has
  no control and its label is gone, after `Esc` it keeps k5

### Requirement: Reset to profile
The system SHALL offer on the MIDI tab, per connected device, an action that replaces the
device's bindings with the roles of its profile; it asks first and says «This cannot be
undone.», `Enter` resets, `Esc` keeps.

#### Scenario: Reset to profile
- **WHEN** the owner clears b7 and learns b3 for "next bank", then selects "reset to
  profile" for the X-Session Pro and confirms the question with Enter
- **THEN** left Cue (b7) opens pad edit again and right Headphone (b3) opens FX

#### Scenario: The reset question says that it has no undo
- **WHEN** the owner selects "reset to profile" for a device
- **THEN** the question says «This cannot be undone.»
