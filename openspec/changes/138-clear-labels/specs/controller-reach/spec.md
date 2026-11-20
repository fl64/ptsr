## MODIFIED Requirements

### Requirement: Current actions have paths from both inputs

Each keyboard action that needs no typed text SHALL have a route from the shipped SMC-PAD profile on each current screen and mode. Pad banks MAY use the Pads list. Tempo MAY use only Pattern transport. Pad and A1 commands MAY use their screen actions. Assign order MAY use the Chop menu. A Chop trim clear MAY use knobs 5 and 6 turned to the ends. An A1 pass undo MAY use the undo cell. Binding changes are keyboard only and have no controller route. Each shipped SMC-PAD sampler action SHALL have a key or key path with the same result. A Chop slice-bound turn SHALL select the same bound as its key. Routes SHALL use production input dispatch.

#### Scenario: Every current keyboard action has a controller route
- **WHEN** the reach test visits each current screen and mode with the shipped keyboard map, SMC-PAD profile and panel map
- **THEN** each non-text keyboard action reaches the same result through raw SMC-PAD MIDI
- **AND** the test reports the screen, mode and action if no controller route exists

#### Scenario: Every current controller action has a keyboard route
- **WHEN** the reach test visits each shipped SMC-PAD button, pad, panel cell and knob role
- **THEN** each sampler action reaches the same result from the keyboard
- **AND** a Chop slice-bound turn selects the same start or end bound as its keyboard route
- **AND** the test reports any action without a key or key path, apart from panel navigation

#### Scenario: Trim clear and A1 pass undo have controller routes
- **WHEN** the reach test checks `chop.region_clear` and `pattern.undo` without their panel cells
- **THEN** knobs 5 and 6 turned to the ends clear the trim, and the undo cell undoes the A1 pass

### Requirement: Input exceptions stay explicit

The only controller-only steps SHALL be PAD BANK selection, its status mark and page navigation within its panel. The only keyboard-only operations SHALL be `pad.rename`, `browser.filter`, a typed new-project name, keymap edits and binding changes. Binding changes SHALL be F10 Learn, learn and forget in F12 MIDI, reset to profile and the SMC-PAD layout update. The controller SHALL give only the control to bind. A new project with a date name SHALL have a controller route. `app.quit` SHALL remain unavailable for MIDI learn. No other exception SHALL enter the inventory without an owner decision.

#### Scenario: Text entry keeps its exceptions
- **WHEN** the reach inventory examines pad naming, file filtering, a typed new-project name, keymap edits and quit
- **THEN** it marks only the four text operations as keyboard-only
- **AND** MIDI learn does not offer `app.quit`

#### Scenario: Binding changes are keyboard only
- **WHEN** the reach inventory examines F10 Learn, learn and forget in F12 MIDI, reset to profile and the SMC-PAD layout update
- **THEN** it marks them as keyboard-only binding changes
- **AND** no panel view shows learn, learn control or forget control

### Requirement: Pickers accept SMC-PAD selection without typed text

In the project picker, SMC-PAD knob 1 SHALL move the selection. Device pad 4 SHALL confirm the selected project, and device pad 1 SHALL cancel. With no project in the list, device pad 4 SHALL read `new project` and SHALL make and open a project with a date name. These modal routes SHALL take priority over the screen behind the picker. A visible question SHALL keep its pad 1 and pad 4 priority. A failed project transition SHALL keep the current project, picker and selection. The picker hint SHALL name `knob 1` and SHALL name pads 1 and 4 with PAD BANK.

#### Scenario: Controller opens a selected project
- **WHEN** the owner opens the project picker, turns knob 1 and presses device pad 4
- **THEN** the selected project opens after its required save succeeds
- **AND** a refused save keeps the old project and the picker selection

#### Scenario: The picker hint names PAD BANK
- **WHEN** the project picker shows after SMC-PAD input
- **THEN** the hint says «PAD BANK + 4 open» and «PAD BANK + 1 close»

### Requirement: Controller can finish learning

With an F10 or MIDI-tab learning choice pending, Enter SHALL save that choice and Esc or device pad 1 SHALL cancel it. With no choice pending, device pad 1 SHALL leave learning. Device pad 4 SHALL always remain available as a control to learn. The answer SHALL not replace the chosen control or run a panel action. The learning prompt SHALL name Enter as the save control.

#### Scenario: Learning answer keeps its chosen control
- **WHEN** the owner selects an action, presses device pad 4 as the control to learn and then presses pad 4 again
- **THEN** the first press becomes the pending control, the second press does not save it, and Enter saves it once
- **AND** a separate pending choice followed by pad 1 cancels without a new choice or screen action
- **AND** pad 1 with no choice pending leaves learning

### Requirement: A refused panel action says why

A panel cell that cannot act in the current state SHALL show a notice with the reason and the place where it works. It SHALL NOT change state.

#### Scenario: Learn on Settings explains the refusal
- **WHEN** the owner presses F10 while Settings shows
- **THEN** a notice says where Learn works and the Settings screen stays unchanged

#### Scenario: Choose on a binding row explains the refusal
- **WHEN** the owner selects an action row in F12 MIDI and presses `choose` from the SMC-PAD
- **THEN** a notice says «Bindings: keyboard only (F10, Enter)» and no learning starts

### Requirement: Learn warns before it replaces a panel cell

A pending Learn binding can fall on an SMC-PAD panel cell. Then the prompt SHALL name the cell by its printed number and its current label before the save.

#### Scenario: Learn names the replaced panel cell
- **WHEN** the PAD BANK hold is off, and the owner learns pad mode on Pads and touches device pad 6 on the PAD BANK layer
- **THEN** the prompt says that the binding replaces panel cell P6 copy pad
