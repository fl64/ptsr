# midi-learn Specification

## Purpose
MIDI learn on the F12 MIDI tab: every action by group (global and per screen), one control per action within a group, saved in the user config over the device profile.

## Requirements

### Requirement: Learn a control for an action
The system SHALL let the owner pick an action on the MIDI tab of the settings screen (F12), wait for a control to be moved and show it pending on the row before it saves anything. The pending row shows what the control drives now: in the group of the row, the action that loses the control, in the taken colour with a warning; in the other groups, each action that the control keeps, in grey, with `stays` (for example `b9 on GLOBAL: bank next · stays`). `Enter` assigns the control to the action and saves it. `Esc` drops the pending control and changes nothing. Another control moved before `Enter` takes the place of the pending one.

#### Scenario: Knob for pad volume
- **WHEN** the owner selects "pad volume", presses Enter, turns a knob, touching a neighbour knob briefly, and presses Enter again
- **THEN** the knob that moved most is assigned, the row shows it, and the knob changes the selected pad volume with soft takeover

#### Scenario: Button for the next bank
- **WHEN** the owner selects "bank next", presses Enter, presses a controller button and presses Enter again
- **THEN** that button switches to the next bank; the control the row had before no longer does

#### Scenario: Cancel and clear
- **WHEN** the owner presses Esc while the program waits for a control, or Backspace on an assigned row
- **THEN** waiting stops without changes, or the binding is removed and the action has no
  control, also when it came from the profile

#### Scenario: Clear a profile role
- **WHEN** left Cue (b7) opens pad edit from the X-Session Pro profile, and the owner
  clears the "open pad edit only" row and restarts the program
- **THEN** b7 does nothing on the pads screen, and the row still has no control

#### Scenario: See what the button leaves before it is saved
- **WHEN** b9 switches the pad mode in the PADS group and turns to the next bank in the GLOBAL group, and on the MIDI tab of F12 the owner selects "pad mark" in PADS, presses Enter and presses b9
- **THEN** b9 stands pending on the "pad mark" row, the "pad mode" row lights in the taken colour with a warning that it loses b9, the line names `b9 on GLOBAL: bank next · stays` in grey, and nothing is saved yet

#### Scenario: Esc on F12 keeps all
- **WHEN** b9 stands pending on a row of the MIDI tab and the owner presses Esc
- **THEN** every binding stays as it was and the config file does not change

### Requirement: Every action can be learned
The system SHALL list on the MIDI tab every action a key can do, grouped as GLOBAL (works on every screen), one group per screen and one group per edit layer (PADS · EDIT, CHOP · EDIT: the page knobs 1–3, the previous and the next parameter, leaving the edit), except quitting the program; the list SHALL scroll so the selected row is always visible. Inside an edit mode the learned edit group of the screen goes first, then the device profile's edit layer, then the screen and the global groups.

#### Scenario: Global action
- **WHEN** the owner learns a button for "help" in GLOBAL
- **THEN** the button opens the help on the pads screen and on the settings screen

#### Scenario: Screen group wins
- **WHEN** the same button is learned for "help" in GLOBAL and for "pad mode" in PADS
- **THEN** on the pads screen it switches the pad mode, on the settings screen it opens the help

#### Scenario: Same knob, two layers
- **WHEN** a knob is learned for "pad volume" in PADS and for "knob 1 of the page" in PADS · EDIT, and the owner turns it with the pad edit closed and then open on the ENV page
- **THEN** it turns the volume first and the attack after

#### Scenario: Outside the edit
- **WHEN** a button is learned only for "next parameter" in PADS · EDIT and the owner presses it with no edit mode open
- **THEN** nothing happens

### Requirement: One control, one action
The system SHALL keep each learned control on a single action per group: assigning it to a new action in the group removes it from the old one in that group. The control keeps its actions in the other groups: on a screen, the group of the screen wins over the global group, so one control can do one thing on the pads screen and another thing on other screens.

#### Scenario: Reassign a button
- **WHEN** a button learned for "bank next" in the GLOBAL group is learned for "pad mode" in the PADS group
- **THEN** on the pads screen it switches the pad mode, on the other screens it still turns to the next bank, and the learn showed `bank next · stays` before the owner pressed Enter

### Requirement: Learned assignments are kept
The system SHALL save the bindings in the user config `~/.config/ptsr/config.toml`
and keep them after a restart; once a device is seeded (input-devices, "Device profiles")
they are its only actions, the profile file is not changed.

#### Scenario: Restart
- **WHEN** the owner learned controls and restarts the program
- **THEN** the controls do the same actions as before the restart

#### Scenario: Cleared stays cleared
- **WHEN** the owner cleared every row of the X-Session Pro and restarts the program
- **THEN** no control of it does anything but name itself on the input debug tab, and the
  profile roles are not copied again

### Requirement: Jump between action groups
The system SHALL move the selection of the MIDI learn list to the previous or next group row with the bank actions (PgUp/PgDn).

#### Scenario: To the chop group
- **WHEN** the MIDI tab is open and the owner presses PgDn until the CHOP group
- **THEN** the CHOP group row is selected

### Requirement: Groups fold
The system SHALL show every group of the MIDI learn list as a row with its name, the number of its actions and how many are learned, and its actions under it only while it is open. The group of the screen the settings were opened from is open, the others are closed; `→` (`L`) or `Enter` on a group row opens it, `←` (`J`) closes the group of the selected row.

#### Scenario: Closed groups
- **WHEN** the owner opens F12 from the pads screen and goes to the MIDI tab
- **THEN** PADS is open, every other group is one row like "CHOP · 31 actions · 2 learned"

#### Scenario: Open a group
- **WHEN** the CHOP row is selected and the owner presses →
- **THEN** the actions of CHOP show under it; ← on one of them closes CHOP and selects its row

### Requirement: Pads and banks in a row
The system SHALL show pads 1–16 and banks 1–8 as one row each with how many of them are learned, and SHALL learn them one after another from that row: every control pressed goes to the next one, a control already taken in the same run is ignored, Esc stops and keeps what is learned, Backspace clears them all.

#### Scenario: Sixteen pads in a row
- **WHEN** the owner selects "pads 1–16", presses Enter and hits 16 controller pads in turn
- **THEN** controller pad k plays pad k, and the row says "16 of 16 learned"

#### Scenario: A double hit
- **WHEN** during the run the owner hits the same controller pad twice
- **THEN** the second hit is ignored and the run waits for another control

#### Scenario: Stop in the middle
- **WHEN** the owner presses Esc after 5 pads
- **THEN** pads 1–5 stay learned and the row says "5 of 16 learned"

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
as a learned assignment of the group of that layer, and the pick moves on to the next element
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

### Requirement: Every pad parameter can be learned
The system SHALL let a control be learned for each pad parameter on its own: volume,
pitch, attack, decay, release, filter, cutoff, reso, start, end. A knob bound to one turns
that parameter of the selected pad on the pads screen, with the edit open or closed, with
the soft takeover, and for the change scope as the keys do.

#### Scenario: The cutoff on a knob
- **WHEN** k5 is learned for the cutoff and the owner turns it with the pad edit closed and then open on the ENV page
- **THEN** both times the cutoff of the selected pad moves, the decay does not

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

### Requirement: Reset to profile
The system SHALL offer on the MIDI tab, per connected device, an action that replaces the
device's bindings with the roles of its profile; it asks first, `Enter` resets, `Esc`
keeps.

#### Scenario: Reset to profile
- **WHEN** the owner clears b7 and learns b3 for "next bank", then selects "reset to
  profile" for the X-Session Pro and confirms the question with Enter
- **THEN** left Cue (b7) opens pad edit again and right Headphone (b3) opens FX
