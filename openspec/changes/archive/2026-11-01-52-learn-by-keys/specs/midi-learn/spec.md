## MODIFIED Requirements

### Requirement: Learn on the screen
The system SHALL turn a learn mode on and off with `F4` on the pads, chop and files
screens. While it is open the top line of the screen says that the mode is on, what is
picked and what to do next, in words; elements with a control carry a label with it,
elements without one carry none. An element is picked by its own key: a keyboard pad key
picks that pad, a key of a screen command picks that command, `PgUp` `PgDn` pick the bank
switch, `I` `K` walk the pad parameters of the panel. The picked element is lit. A control
moved on the controller is not saved at once: the top line asks whether the control should
drive the picked element; when the control already drives another element of the layer in
sight, that element is lit too and the line names it as the one that loses the control.
`Enter` saves the binding, as a learned assignment of the learn list, `Esc` drops it.
`Backspace` asks the same way before it clears the picked element's control. `F4` leaves
the mode. While the mode is open no pad sounds and no command acts; the keys keep what
they do outside it.

#### Scenario: Pick by the pad key
- **WHEN** the mode is open on the pads screen and the owner presses pad key Q
- **THEN** pad Q is lit, no sound plays, and the top line says pad Q is picked and to move a control

#### Scenario: A control asks first
- **WHEN** pad Q is picked and the owner hits controller pad 5
- **THEN** the top line asks whether it should play pad Q, and nothing is saved until `Enter`; after `Enter` controller pad 5 plays pad Q

#### Scenario: A taken control
- **WHEN** knob k5 turns the volume, the owner picks the cutoff with `I` `K` and turns k5
- **THEN** the volume row lights too and the line says k5 leaves the volume; `Esc` keeps k5 on the volume, `Enter` moves it to the cutoff

#### Scenario: A command by its key
- **WHEN** the mode is open on the chop screen and the owner presses `M`
- **THEN** "cut" is picked and lit, and no cut is made

#### Scenario: Silent while learning
- **WHEN** the mode is open and the owner presses a pad key and the pad mode key `H`
- **THEN** no pad sounds, the mode of no pad changes, and `H` picks "pad mode"

#### Scenario: Clear asks too
- **WHEN** the cutoff, bound to k5, is picked and the owner presses `Backspace` and then `Enter`
- **THEN** the cutoff has no control and its label is gone

## REMOVED Requirements

### Requirement: The walk goes by zones
**Reason**: the owner found the walk unclear; an element is picked by its own key (52).
**Migration**: a pad key, a command key, `PgUp` `PgDn` pick; `I` `K` walk the parameters.

## ADDED Requirements

### Requirement: Every pad parameter can be learned
The system SHALL let a control be learned for each pad parameter on its own: volume,
pitch, attack, decay, release, filter, cutoff, reso, start, end. A knob bound to one turns
that parameter of the selected pad on the pads screen, with the edit open or closed, with
the soft takeover, and for the change scope as the keys do.

#### Scenario: The cutoff on a knob
- **WHEN** k5 is learned for the cutoff and the owner turns it with the pad edit closed and then open on the ENV page
- **THEN** both times the cutoff of the selected pad moves, the decay does not
