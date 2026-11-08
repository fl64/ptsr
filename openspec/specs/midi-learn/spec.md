# midi-learn Specification

## Purpose
MIDI learn on the F12 MIDI tab: every action by group (global and per screen), one control per action within a group, saved in the user config over the device profile.

## Requirements

### Requirement: Learn a control for an action
The F12 MIDI tab SHALL let the owner select an action and wait for a moved control.
The row SHALL show that control pending without saving.
`Enter` SHALL assign and save it.
`Esc` SHALL drop it without changes.
Another control moved before `Enter` SHALL replace the pending control.

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

### Requirement: MIDI tab pending assignment warnings

The pending row SHALL show the control's current actions.
Within the row's group, the losing action SHALL use the taken colour and a warning.
In other groups, retained actions SHALL appear grey with `stays`, for example `b9 on GLOBAL: bank next · stays`.

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

`F10` SHALL toggle screen learn on Pads, Chop, Files and FX.
A learning block in the fixed three-row help band SHALL name the mode, selected element, control and next step.
It SHALL name action, layer and page, including `cutoff · PADS · EDIT · FILTER`.
Bound elements SHALL carry control labels. Unbound elements SHALL carry none.
The selected element SHALL remain lit and show its control or `none`.

#### Scenario: Pick by the pad key
- **WHEN** the mode is open on the pads screen and the owner presses pad key Q
- **THEN** pad Q is lit and no sound plays
- **AND** the learning block identifies pad Q and asks the owner to move a control

#### Scenario: The binding is shown
- **WHEN** pad Q plays from controller pad 5 and the owner picks pad Q
- **THEN** the label of pad Q and the learning block say `pad 5`
- **AND** a pad with no control says `none`

### Requirement: Screen learn waits for confirmation

A moved control SHALL appear beside the old one in the selected label and learning block, in pending colour (`pad 5 → k3`).
It SHALL NOT bind immediately.
`Enter` SHALL bind it in the selected layer's learned group and advance to the next pad, parameter or command.
Learn SHALL ignore that control until another control moves or a key selects an element.

#### Scenario: A turn waits for Enter
- **WHEN** the owner selects pad Q, which has controller pad 5, and hits controller pad 6
- **THEN** the label says `pad 5 → pad 6` with `pad 6` in the pending colour
- **AND** pad 5 still plays pad Q
- **AND** after `Enter` pad 6 plays pad Q and the program selects pad W
- **AND** another pad 6 hit does not show as pending for pad W

#### Scenario: The panel row shows the pending control
- **WHEN** the owner selects pitch with `I` `K`, with cc 24 bound, and turns cc 26
- **THEN** the pitch row of the pad panel says `◂ cc 24 → cc 26`, with `cc 26` in the pending colour
- **AND** the pitch still follows cc 24

### Requirement: Screen learn recalls the last action

`F10` within 5 s of a key action, pad hit or value step SHALL select that action in its original layer and page.
A value step SHALL select its changed parameter.
The learning block SHALL identify a last-action selection.
After a refused control, it SHALL explain that `I` `K`, pad keys or command keys select another element.

#### Scenario: Do, then F10
- **WHEN** the pad edit is open on FILTER, the owner steps the cutoff with `L`, presses `F10` within 5 s and turns k5
- **THEN** the learning block says `cutoff · PADS · EDIT · FILTER`, and the cutoff row says `none → k5`
- **AND** after `Enter` k5 turns the cutoff in the edit, but keeps its previous action with the edit closed

#### Scenario: The line says where the pick came from
- **WHEN** the owner presses `F10`, then `Enter` to pick `edit pad`, and turns a knob
- **THEN** the learning block explains that `edit pad` needs a button and that `I` `K` select a parameter
- **WHEN** the owner presses `PgDn`, then `F10` at once
- **THEN** the learning block identifies `next bank` as the last action

#### Scenario: Too late for the last action
- **WHEN** the owner presses `PgDn` and presses `F10` 10 s later
- **THEN** the mode opens with nothing picked, as without an action

### Requirement: Screen learn checks control types

Parameters SHALL accept knobs/faders that drive them directly.
Commands SHALL accept buttons/controller pads.
Wrong control types SHALL NOT become pending.
The learning block SHALL explain why: parameters have values to turn, buttons only press, and knobs repeat commands on every tick.

#### Scenario: A button for a parameter
- **WHEN** the owner selects cutoff and presses controller button b6
- **THEN** nothing is pending, and the learning block explains that cutoff needs a knob or fader instead of a button

### Requirement: Screen learn preserves layer scope

An element selected in edit SHALL bind to that screen's edit group and work while edit is open.
Otherwise, it SHALL bind to the screen group.
A control's losing element in the same layer SHALL light in taken colour with a warning in the learning block.
`Enter` SHALL move it.
Other-layer assignments SHALL appear grey in that block without warning, because layers act at different times.

#### Scenario: Taken in this layer
- **WHEN** k5 turns resonance in pad edit and the owner learns k5 for cutoff in the same edit
- **THEN** resonance lights in taken colour, and the learning block warns that resonance loses k5
- **AND** `Esc` keeps k5 on resonance, while `Enter` moves it to cutoff

#### Scenario: Bound in another layer
- **WHEN** k5 turns pad volume and the owner learns k5 for cutoff in pad edit
- **THEN** the learning block names `k5 on PADS: volume` in grey without a warning
- **AND** after `Enter` k5 turns volume outside the edit and cutoff inside the edit

### Requirement: Screen learn pending control replacement

`Esc` SHALL drop a pending control while keeping the old binding and selected element.
Another moved control SHALL replace the pending control.
`F10` SHALL leave learn and drop pending input.

#### Scenario: Esc keeps the old one
- **WHEN** a control stands pending for cutoff and the owner presses `Esc`
- **THEN** cutoff keeps its old control and stays selected, without a pending colour

#### Scenario: Another control takes its place
- **WHEN** k3 stands pending for cutoff and the owner turns k4
- **THEN** k4 stands pending in place of k3

#### Scenario: A knob is not a button
- **WHEN** the owner selects `next bank` and turns a knob
- **THEN** nothing is pending
- **AND** the learning block explains that this action needs a button because a knob would repeat the action

#### Scenario: A taken control
- **WHEN** k5 turns volume and the owner selects cutoff with `I` `K` and turns k5
- **THEN** volume lights, and the learning block says k5 leaves volume
- **AND** `Esc` keeps k5 on volume, while `Enter` moves it to cutoff

### Requirement: Screen learn key selection is silent

Each element's own key SHALL select it: pad keys select pads, command keys select commands, and `PgUp`/`PgDn` select bank switches.
`I`/`K` SHALL walk pad-panel parameters.
While learn is open, no pad SHALL sound and no command SHALL execute.
Outside learn, keys SHALL keep their existing behavior.

#### Scenario: A command by its key
- **WHEN** the mode is open on the chop screen and the owner presses `M`
- **THEN** "cut" is picked and lit, and no cut is made

#### Scenario: Silent while learning
- **WHEN** the mode is open and the owner presses a pad key and the pad mode key `H`
- **THEN** no pad sounds, the mode of no pad changes, and `H` picks "pad mode"

### Requirement: Screen learn clearing needs confirmation

`Backspace` SHALL make "none" pending.
`Enter` SHALL clear the selected element's control, and `Esc` SHALL keep it.

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

### Requirement: Learn the keymap cleanup actions

F12 MIDI SHALL list `pad.copy` in PADS and `chop.assign_order`/`chop.hits_back` in CHOP.
F10 SHALL select them with Ctrl+C on Pads, Tab/Shift+H on Chop, or effective owner keys, without executing them.
They SHALL accept controller buttons/pads, not absolute knobs.

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

### Requirement: Learned cleanup command behavior

Learned copy SHALL take the selected pad in hand and accept keyboard/controller pads as targets.
Assign-order and hits-back SHALL keep their Chop mode guards.
Old `edit.param_next` bindings SHALL keep switching assignment order.

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

### Requirement: Cleanup bindings keep saved assignments

Saved cleanup-command bindings SHALL survive restart.
The cleanup SHALL NOT reset or reassign existing owner bindings or X-Session defaults.

#### Scenario: Cleanup learned bindings survive restart
- **WHEN** the owner saves bindings for all three new actions and restarts
- **THEN** they keep their assigned controls and all previous learned bindings stay unchanged

#### Scenario: Cleanup does not reset X Session controls
- **WHEN** the cleanup is installed with an initialized or owner-modified X-Session profile
- **THEN** all its controls keep their roles without a reset or new unsafe direct command

### Requirement: Learning preserves the common status

Screen learning SHALL keep the shared status row and its safety indicators visible.
Its learning block SHALL stay inside the existing three-row help band without moving the title, grid or notice row.
The block SHALL distinguish selected context, pending or refusal detail, and working next-step controls.
Long labels SHALL shorten before next-step controls disappear.
Learning input, silence, confirmation and saved assignments SHALL remain unchanged.

#### Scenario: Learning keeps safety and next steps visible
- **WHEN** keyboard or raw MIDI opens screen learning while save state, recording or audio warnings are present
- **THEN** the status retains its applicable safety indicators at 80×26 and 120×40
- **AND** the learning block shows its selected context and working next steps without moving the grid

### Requirement: Learn transport controls

F12 MIDI SHALL list `transport.toggle`, `metronome.toggle`, `transport.tempo_up`, `transport.tempo_down` and their `_fine` forms as GLOBAL button actions. It SHALL list `transport.tempo` as a GLOBAL knob parameter. Existing confirmation, clear, layer precedence, input ordering and saved-binding rules SHALL apply. Press actions SHALL execute once, not again on release.

#### Scenario: Raw MIDI buttons control transport and metronome
- **WHEN** raw MIDI presses and releases reach learned transport and metronome buttons through the production profile and dispatch
- **THEN** their actions match the keyboard path and releases do not repeat them

#### Scenario: Raw MIDI tempo buttons match keyboard steps
- **WHEN** raw MIDI uses learned coarse or fine tempo buttons
- **THEN** each press changes tempo by 1 or 0.1 BPM with the same bounds as keyboard input

#### Scenario: Transport learn selects without playing
- **WHEN** the owner selects new transport controls in F12 MIDI or screen learn and moves a control
- **THEN** the binding remains pending until confirmation and selection does not execute the transport command
- **AND** a knob cannot bind to a button command through screen learn

### Requirement: Absolute tempo knob

A learned tempo knob SHALL map MIDI values 0–127 to 40.0–300.0 BPM, rounded to 0.1 BPM. Existing jump, pickup and scale takeover SHALL apply separately per device and control. A keyboard tempo change or project switch SHALL use the current tempo for takeover. Successful changes SHALL show the existing value popup with BPM and its range.

#### Scenario: Tempo knob maps its full range
- **WHEN** a learned absolute tempo knob sends raw MIDI values 0, 64 and 127 through production dispatch
- **THEN** tempo becomes 40.0, 171.0 and 300.0 BPM in jump mode
- **AND** the value popup identifies tempo in BPM

#### Scenario: Tempo pickup remains separate per controller
- **WHEN** two learned tempo knobs use pickup and the keyboard changes tempo between their movements
- **THEN** each knob must reach the current tempo under the existing per-control takeover rules
- **AND** moving one control does not unlock the other

### Requirement: Transport bindings preserve controller roles

Adding transport SHALL NOT reset, seed again, reassign or rewrite existing X-Session roles or owner bindings. Saved transport bindings SHALL survive restart. Transport commands from a previous project SHALL NOT change the new project's clock. The incomplete SMC-PAD template SHALL remain unverified.

#### Scenario: New transport bindings survive restart without a profile reset
- **WHEN** the owner saves transport bindings and restarts with existing X-Session assignments
- **THEN** the new bindings and all unchanged assignments retain their roles without reseeding

#### Scenario: Late transport commands stay in their original project
- **WHEN** a queued transport or tempo command belongs to the old project and a new project opens
- **THEN** that command does not start, stop or change the new project's clock
