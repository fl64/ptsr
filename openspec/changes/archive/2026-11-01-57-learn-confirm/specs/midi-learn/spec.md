## MODIFIED Requirements

### Requirement: Learn on the screen
The system SHALL turn a learn mode on and off with `F10` on the pads, chop and files
screens. While it is open the top line of the screen says that the mode is on, what is
picked, its control and what to do next, in words; elements with a control carry a label
with it, elements without one carry none. An element is picked by its own key: a keyboard
pad key picks that pad, a key of a screen command picks that command, `PgUp` `PgDn` pick
the bank switch, `I` `K` walk the pad parameters of the panel. `F10` pressed within 5 s
after an action was done (a key, a pad hit, a value step) opens the mode with that action
picked, in the layer and on the page where it was done: a value step picks the parameter
it changed. The top line says when the pick is the last thing done, and a refused control
says that `I` `K`, a pad or a command key pick another. The picked element is lit and its label shows its control, or "none"; the top
line names the action, its layer and page (`cutoff · PADS · EDIT · FILTER`).
A binding goes to the layer where the element was picked: with the edit open, to the edit
group of the screen, and works while the edit is open; with it closed, to the screen's
group. A knob for a parameter drives that parameter directly. A parameter takes a knob or
a fader, a command takes a button or a pad: the other kind is not shown as pending, and the
top line says why in words — a parameter has a value to turn and a button only presses; a
command acts once per press and a knob would repeat it on every tick.
A control moved on the controller does not bind at once: it stands next to the old one
in the label of the picked element and in the top line, in the pending colour
(`pad 5 → k3`). Another control moved before the answer takes its place. `Enter` binds it
as a learned assignment of the screen's group, and the pick moves on to the next element
(the next pad, the next parameter, the next command); the control just bound is not heard
by the mode until another control moves or an element is picked by its key. `Esc` drops
the pending control, the old one stays, the pick stays. A control that already drives
another element in the same layer lights that element in the "taken" colour, and the top
line warns that it loses the control; `Enter` moves it. A control bound in another layer
is named in the top line in grey, with no warning: the layers work at different times. A command takes a button or a pad of the
controller: a knob turned for it is not shown as pending and the top line says that a
button is needed and why. `Backspace` puts "none" as pending; `Enter` clears the picked element's
control, `Esc` keeps it. `F10` leaves the mode, a pending control is dropped. While the
mode is open no pad sounds and no command acts; the keys keep what they do outside it.

#### Scenario: Pick by the pad key
- **WHEN** the mode is open on the pads screen and the owner presses pad key Q
- **THEN** pad Q is lit, no sound plays, and the top line says pad Q is picked and to move a control

#### Scenario: The binding is shown
- **WHEN** pad Q plays from controller pad 5 and the owner picks pad Q
- **THEN** the label of pad Q and the top line say `pad 5`; a pad with no control says `none`

#### Scenario: A turn waits for Enter
- **WHEN** pad Q is picked, bound to controller pad 5, and the owner hits controller pad 6
- **THEN** the label says `pad 5 → pad 6` with `pad 6` in the pending colour, pad 5 still
  plays pad Q; after `Enter` pad 6 plays pad Q, pad W is picked next, and pad 6 hit again
  does not show as pending for pad W

#### Scenario: The panel row shows the pending control
- **WHEN** the pitch, bound to cc 24, is picked with `I` `K` and the owner turns cc 26
- **THEN** the pitch row of the pad panel says `◂ cc 24 → cc 26`, `cc 26` in the pending
  colour, and the pitch still follows cc 24

#### Scenario: Do, then F10
- **WHEN** the pad edit is open on the FILTER page, the owner steps the cutoff with `L`,
  presses `F10` within 5 s and turns k5
- **THEN** the top line says `cutoff · PADS · EDIT · FILTER`, the cutoff row says
  `none → k5`; after `Enter` k5 turns the cutoff while the pad edit is open, and with the
  edit closed k5 keeps what it had on the pads screen

#### Scenario: The line says where the pick came from
- **WHEN** the owner presses `F10`, then `Enter` (it picks "edit pad", the action of its
  key), and turns a knob
- **THEN** the top line says "edit pad" needs a button and why, and that `I` `K` pick a
  parameter
- **WHEN** the owner presses `PgDn`, then `F10` at once, and turns a knob
- **THEN** the top line also says "next bank" is the last thing done

#### Scenario: Too late for the last action
- **WHEN** the owner presses `PgDn` and presses `F10` 10 s later
- **THEN** the mode opens with nothing picked, as without an action

#### Scenario: A button for a parameter
- **WHEN** the cutoff is picked and the owner presses controller button b6
- **THEN** nothing is pending, and the top line says the cutoff needs a knob or a fader
  because it has a value to turn and a button only presses

#### Scenario: Taken in this layer
- **WHEN** k5 turns the resonance in the pad edit and the owner learns k5 for the cutoff in
  the pad edit
- **THEN** the resonance row lights in the taken colour and the top line warns that the
  resonance loses k5; `Esc` keeps it, `Enter` moves k5 to the cutoff

#### Scenario: Bound in another layer
- **WHEN** k5 turns the pad volume on the pads screen and the owner learns k5 for the
  cutoff in the pad edit
- **THEN** the top line names `k5 on PADS: volume` in grey with no warning; after `Enter`
  k5 turns the volume with the edit closed and the cutoff with it open

#### Scenario: Esc keeps the old one
- **WHEN** a control stands pending for the cutoff and the owner presses `Esc`
- **THEN** the cutoff keeps its old control, the cutoff stays picked, and the pending
  colour is gone

#### Scenario: Another control takes its place
- **WHEN** k3 stands pending for the cutoff and the owner turns k4
- **THEN** k4 stands pending in place of k3

#### Scenario: A knob is not a button
- **WHEN** "next bank" is picked and the owner turns a knob
- **THEN** nothing is pending and the top line says a button is needed, because the next
  bank acts once per press and a knob would repeat it on every tick

#### Scenario: A taken control
- **WHEN** knob k5 turns the volume, the owner picks the cutoff with `I` `K` and turns k5
- **THEN** the volume row lights too and the line says k5 leaves the volume; `Esc` keeps k5 on the volume, `Enter` moves it to the cutoff

#### Scenario: A command by its key
- **WHEN** the mode is open on the chop screen and the owner presses `M`
- **THEN** "cut" is picked and lit, and no cut is made

#### Scenario: Silent while learning
- **WHEN** the mode is open and the owner presses a pad key and the pad mode key `H`
- **THEN** no pad sounds, the mode of no pad changes, and `H` picks "pad mode"

#### Scenario: Clear waits for Enter
- **WHEN** the cutoff, bound to k5, is picked and the owner presses `Backspace`
- **THEN** the label says `k5 → none` in the pending colour; after `Enter` the cutoff has
  no control and its label is gone, after `Esc` it keeps k5

## ADDED Requirements

### Requirement: Edit parameters on the MIDI tab
The system SHALL list on the MIDI tab of F12, in PADS · EDIT, the pad parameters (volume,
pitch, attack, decay, release, filter, cutoff, reso, start, end) as knob rows, and in
CHOP · EDIT the chop parameters, so a knob bound to a parameter in the edit layer (by F10)
shows on its row there and can be relearned or cleared with `Backspace`, as every row.

#### Scenario: The cutoff in the edit on F12
- **WHEN** k5 was bound to the cutoff with the pad edit open and the owner opens F12, the
  MIDI tab, the PADS · EDIT group
- **THEN** the "pad cutoff" row of the group says k5, and `Backspace` on it leaves k5
  turning nothing in the edit
