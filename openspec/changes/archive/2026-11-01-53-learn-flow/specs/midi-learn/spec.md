## MODIFIED Requirements

### Requirement: Learn on the screen
The system SHALL turn a learn mode on and off with `F4` on the pads, chop and files
screens. While it is open the top line of the screen says that the mode is on, what is
picked and what to do next, in words; elements with a control carry a label with it,
elements without one carry none. An element is picked by its own key: a keyboard pad key
picks that pad, a key of a screen command picks that command, `PgUp` `PgDn` pick the bank
switch, `I` `K` walk the pad parameters of the panel. The picked element is lit. A control
moved on the controller that drives nothing on this screen is bound to the picked element
at once, as a learned assignment of the screen's group, and the pick moves on to the next
element (the next pad, the next parameter, the next command); the control just bound is
not heard by the mode until another control moves or an element is picked by its key. A control that already drives another
element of the screen is not bound at once: that element is lit too, the top line names
it as the one that loses the control, `Enter` binds, `Esc` drops it. A command takes a
button or a pad of the controller: a knob turned for it is not bound and the top line
says that a button is needed.
`Backspace` asks the same way before it clears the picked element's control. `F4` leaves
the mode. While the mode is open no pad sounds and no command acts; the keys keep what
they do outside it.

#### Scenario: Pick by the pad key
- **WHEN** the mode is open on the pads screen and the owner presses pad key Q
- **THEN** pad Q is lit, no sound plays, and the top line says pad Q is picked and to move a control

#### Scenario: A free control binds and moves on
- **WHEN** pad Q is picked and the owner hits controller pad 5, which drives nothing
- **THEN** controller pad 5 plays pad Q at once, pad W is picked next, and controller pad 5 hit again does not bind pad W

#### Scenario: A knob is not a button
- **WHEN** "next bank" is picked and the owner turns a knob
- **THEN** nothing is bound and the top line says a button is needed

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

### Requirement: A binding goes to the layer in sight
**Reason**: a binding made with the pad edit open worked only in the edit, and the owner
could not tell (53).
**Migration**: every binding goes to the screen's group; the page knobs of the edit are
learned in F12.
