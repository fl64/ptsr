## ADDED Requirements

### Requirement: The list buttons follow the current SMC-PAD mode

The shipped LEFT and RIGHT buttons SHALL move the main list of the current mode.
They SHALL select pad banks on Pads, edit pages in pad edit and bar pages in Pattern.
They SHALL select slices on Chop, tabs in Settings and slots in FX.
On Files, LEFT SHALL select the parent folder. RIGHT SHALL enter the selected folder.
A selected file SHALL keep its current Load and Take paths.
An active picker or input overlay SHALL retain its input priority.
A held repeat SHALL follow the list bounds.

#### Scenario: SMC-PAD list buttons name and move the same list
- **WHEN** the shipped buttons act on Pads, pad edit, Pattern, Chop, FX or Settings
- **THEN** each button moves its named list in the direction shown
- **AND** a turn or a hold cannot bypass the list bounds

#### Scenario: File list buttons navigate folders
- **WHEN** the owner turns bank 1 knob 1 to a file and presses RIGHT
- **THEN** the preview target and pad assignment stay the same
- **AND** RIGHT enters the folder after knob 1 selects it, and LEFT returns to its parent

### Requirement: Bank 1 provides position and page values

Bank 1 knob 1 SHALL move the position. It SHALL scroll projects only in the picker.
Knobs 2–7 SHALL change the page values in the design.
Pads and pad edit SHALL use the current inspector page positions.
Unavailable values SHALL stay dim and inactive. Knob 8 SHALL change master volume.
Current input owners SHALL keep their priority.

#### Scenario: Pad inspector and editor share knob positions
- **WHEN** the owner selects each Pad page and turns its assigned bank 1 knobs
- **THEN** the same knobs change the same values in the inspector and editor
- **AND** an unavailable role changes no value and a different screen uses its own page

#### Scenario: Knob 1 moves the current position
- **WHEN** the owner turns bank 1 knob 1 on Pattern, Files, Settings, FX or a picker
- **THEN** the turn moves the current step, item, parameter or row for that mode
- **AND** project selection changes only while its picker owns input

#### Scenario: Chop knobs move two separate bound pairs
- **WHEN** the owner turns bank 1 knobs 3–6 on Chop or Chop edit
- **THEN** each turn changes only the named slice or region edge as the keyboard path does
- **AND** it keeps the current limit, link and Undo rules

#### Scenario: Slow Chop zoom uses smaller input steps
- **WHEN** the owner sends four isolated movement steps on bank 1 knob 2
- **THEN** the zoom changes by one current keyboard zoom step
- **AND** a reverse turn cancels pending movement before it applies an opposite zoom step
- **AND** fast turns retain acceleration

### Requirement: Bank 2 follows the selected pad on each screen

The shipped bank 2 roles SHALL be volume, cutoff, pitch, decay, send 1 and send 2 on knobs 1–6.
Knob 7 SHALL reserve the pan role. It SHALL stay dim until pan is available.
Knob 8 SHALL change master volume through the same action as bank 1 knob 8.
The effect sends SHALL use the selected pad and the existing send limits.
The bank 2 roles SHALL retain their meaning through a screen change.
The program SHALL keep owner bindings and current input overlay rules.

#### Scenario: Bank 2 supplies the same live pad controls on every screen
- **WHEN** the owner selects a pad and turns bank 2 knobs 1–7 across screens and panel pages
- **THEN** available roles change the named value of that pad
- **AND** send 1 and send 2 change different slots and the dim pan role has no action

#### Scenario: Both banks pin master to the top right knob
- **WHEN** the owner turns physical knob 8 in bank 1 and then in bank 2
- **THEN** both turns change the same master volume
- **AND** neither turn changes a bank, pad parameter or tempo

### Requirement: The shipped SMC-PAD tempo knob is local to Pattern transport

Pattern transport SHALL provide tempo, Bars, Quantize and Strength. Swing SHALL stay dim.
A slow tempo step SHALL change 0.1 BPM. A fast turn SHALL support 1 BPM steps.
The shipped map SHALL restrict tempo knobs to Pattern transport.
The Pattern context route SHALL reserve a dim Tap Tempo cell.
Keyboard and learned tempo actions SHALL keep their routes.

#### Scenario: Tempo turns act only on the Pattern transport page
- **WHEN** the owner turns the tempo position on the Pattern transport page
- **THEN** the current tempo handler changes the same value as its keyboard path
- **AND** that knob on Chop, Files, Settings or another page cannot change tempo

### Requirement: Panel buttons select context pages

The shipped More, Next and Main cells SHALL move between panel pages.
A bank 1 knob turn SHALL keep its screen role while the panel shows.
A panel page change SHALL update the visible cell labels before a later cell acts.
More and one action cell SHALL be sufficient for a first context page action.

#### Scenario: Next selects a later context page
- **WHEN** the panel shows a context page and the owner turns bank 1 knob 6
- **THEN** the turn keeps its screen role and the panel page stays the same
- **AND** Next selects the later context page before a cell acts once

## MODIFIED Requirements

### Requirement: Current actions have paths from both inputs

Each current non-text keyboard action SHALL have a production route from the shipped SMC-PAD map.
Tempo may use only Pattern transport. Pad bank changes may use the Pads screen list.
Each shipped controller action SHALL have a keyboard route with the same result.
Chop slice bounds SHALL select the same bound as their keyboard routes.

#### Scenario: Every current keyboard action has a controller route
- **WHEN** the reach test visits each screen and mode with the shipped maps
- **THEN** it finds the controller route for each current non-text action, including the local tempo route
- **AND** the test reports the screen, mode and action if it finds no route

#### Scenario: Every current controller action has a keyboard route
- **WHEN** the reach test visits each effective control in the shipped MIDI map
- **THEN** each sampler action has a keyboard route with the same result
- **AND** either route to a slice bound selects the same bound

### Requirement: A1 controls use the shipped pattern encoder context

While A1 controls are open, the shipped Pattern roles SHALL take priority over underlying screen roles.
Bank 1 knob 1 SHALL select the parameter. The page knobs SHALL change their named values.
The existing keyboard routes SHALL keep their bounds and results.
Closing A1 controls SHALL restore the underlying screen roles.
Other devices, master volume, panel cells and held releases SHALL keep their routes.
Input overlays SHALL keep priority. The Controller tab SHALL show the current roles.

#### Scenario: A1 encoders follow the open controls on every screen
- **WHEN** separate keyboard and raw MIDI runs open A1 controls and change Bars, Quantize or Strength
- **THEN** both runs reach the same settings and bounds
- **AND** knob 1 selects the current parameter and the visible map names its role
- **AND** closing A1 controls restores the underlying roles

### Requirement: A1 encoder bindings survive profile seeding

The Pattern roles SHALL have a separate learned group.
The owner's Pattern bindings SHALL take priority over the shipped roles.
A new SMC-PAD profile SHALL seed the group.
An existing profile SHALL follow the saved layout update rules, including for an empty group.
Learning or removing a binding SHALL retain the effective result after restart.

#### Scenario: A1 encoder bindings survive profile seeding and owner changes
- **WHEN** a new or existing SMC-PAD profile loads and the owner replaces or removes a Pattern binding
- **THEN** the saved choice takes priority while A1 controls are open
- **AND** unrelated groups and other devices keep their assignments

#### Scenario: A1 encoder context preserves input owners and held pads
- **WHEN** an input overlay opens over A1 controls or a held Gate crosses their close
- **THEN** the active input owner keeps priority and no A1 setting changes behind it
- **AND** the Gate release stops its original pad

### Requirement: Chop threshold has matching keyboard steps

On Chop and Chop edit, Ctrl+Alt+Up SHALL increase the hit threshold by 1 dB.
Ctrl+Alt+Down SHALL decrease it by 1 dB.
The keys SHALL keep their existing threshold actions.
An isolated bank 1 knob 7 step SHALL use the same threshold interval.
Fast turns SHALL keep their current acceleration.
The threshold SHALL stay within -40 to 0 dB.

#### Scenario: Keyboard and encoder set the same Chop threshold
- **WHEN** keyboard and raw MIDI runs step the threshold in both directions at least 100 ms apart
- **THEN** each run reaches the same threshold, cuts, linked pads and Undo state
- **AND** an attempt to pass a bound creates no Undo entry

#### Scenario: Fast encoder turns keep acceleration
- **WHEN** bank 1 knob 7 sends two movement steps within 40 ms on Chop
- **THEN** the existing MIDI speed rule accelerates the second step
- **AND** two separate keyboard presses still change the threshold by exactly 2 dB

#### Scenario: Threshold keys respect owner bindings and input owners
- **WHEN** an owner replaces or removes a threshold key, or an input overlay owns input
- **THEN** the owner entry or active mode keeps priority

### Requirement: Pads have a silent keyboard selection route

On Pads and pad edit, Alt+Left and Alt+Right SHALL select the adjacent pad in the same row.
On Pads, the shipped bank 1 knob 1 SHALL use this silent route.
Selection SHALL stop at a row edge. It SHALL preserve the target of each held pad.
A key repeat SHALL not select another pad. Other screens SHALL retain their Alt-arrow routes.
An owner binding or active overlay SHALL keep priority.

#### Scenario: Keyboard selects a neighboring pad without sound
- **WHEN** the owner uses Alt+Left or Alt+Right on Pads or pad edit
- **THEN** the selected pad matches the corresponding bank 1 knob 1 move on Pads
- **AND** the move stops at a row edge without pad sound, a changed value or an ended splash
- **AND** a key repeat does not move again and a held Gate keeps its original release

#### Scenario: Silent selection respects owner bindings and overlays
- **WHEN** an owner changes an Alt-arrow key or an overlay owns it
- **THEN** the owner entry or overlay acts first and the selected pad stays unchanged
- **AND** Chop keeps its current Alt-arrow slice-bound route

### Requirement: Controller routes keep short press paths

Frequent actions SHALL have a direct button, pad, knob or first context page route.
The route table SHALL list the route and counts for each User Story.
From the visible main page, the first context page SHALL take two presses.
Each later context page SHALL add one Next press.
From a hidden panel, count PAD BANK and a reveal hit when the shortcut misses.
Count knob turns, target pad hits and modal answers separately.

#### Scenario: First entry and repeated commands have separate counts
- **WHEN** the owner enters a context page from a hidden or visible panel
- **THEN** only the hidden count includes PAD BANK and a reveal hit when needed
- **AND** each Next press counts once and a knob turn counts as zero presses

#### Scenario: Every story has a recorded controller path
- **WHEN** the reach test reads each action story in User Stories
- **THEN** the route table gives an SMC-PAD path and count
- **AND** observation-only, text-only and other-device stories name their exception

### Requirement: Added context cells provide missing actions

Each view SHALL keep its main page, first context page and fixed bottom row.
The common and view-specific cells below SHALL keep their existing actions.
Current keyboard and MIDI learn actions SHALL remain available.

| View | Page | Cells in order |
| --- | --- | --- |
| Every view | 2 | Output record, transport, take, project picker, save, pad grid, learn, A1 undo, A1 controls, exit |
| Every view | 3 | `pattern.erase`, pattern record |
| Pads | 4 | Previous grid view, next grid view, empty rows, step edit |
| Chop and Chop edit | 4 | Hits back, `marker.delete`, `slice.end_delete`, assign order |

#### Scenario: Added cells reach their current actions
- **WHEN** the owner uses a listed cell where its action is available
- **THEN** the action has the same result as its keyboard path
- **AND** another mode does not show it as available if it has no effect there

## REMOVED Requirements

### Requirement: Context page selection preserves the screen controls

**Reason**: Knob 6 now changes a page value. Discrete page moves use cells.
**Migration**: Use More, Next and Main as specified in "Panel buttons select context pages".
