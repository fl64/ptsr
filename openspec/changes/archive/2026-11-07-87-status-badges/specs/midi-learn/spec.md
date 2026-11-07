## MODIFIED Requirements

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

## ADDED Requirements

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
