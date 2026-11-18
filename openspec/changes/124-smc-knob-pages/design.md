# Design

## Inputs and scope

Use the measured Master port map from [change 91](../archive/2026-11-16-91-smc-pad-profile/).
CC 30–37 select bank 1 roles. CC 38–45 select bank 2 roles.
The port has no separate KNOB BANK message. The owner reports that the knobs have
no detents. Use buttons for lists that have discrete pages.
Use the hardware positions for the strip, panel pair checks and controller tests.

Use the shared action path for each change. Use the same page data for routing
and for visible roles. Keep explicit owner bindings, including disabled controls.
The tables show the shipped roles. A binding can change the effective role.
During a question, a picker, text entry, Learn or Help, apply its input rules first.

## Bank 1 and the two list buttons

K1 means physical knob 1. The last column names the button pair in the help.
All other bank 1 roles stay dim when the page has no value for them.
K8 is Master on every page. The panel has its own More, Next and Main cells.

| Screen or mode | K1 | LEFT and RIGHT | Help label |
| --- | --- | --- | --- |
| Pads | Silent neighboring pad in the row | Previous and next pad bank | Bank |
| Pad edit | Parameter on the page | Previous and next pad edit page | Page |
| Pattern | Step in the selected row | Previous and next bar page | Bar |
| Pattern controls | Parameter on the page | Previous and next parameter page when available | Page |
| Chop and Chop edit | Cursor | Previous and next slice | Slice |
| Files | File or folder | Parent folder and enter selected folder | Folder |
| Settings | Row | Previous and next tab | Tab |
| FX | Parameter in the slot | Previous and next FX slot | Slot |
| Project picker | Project item | Existing picker route | Project |
| Paths folder picker | Folder item | Parent folder and enter selected folder | Folder |

Create the Pattern transport page in Pattern controls. The values use the
current tempo, Bars, Quantize and Strength handlers. Dim swing until it exists.
An isolated tempo movement uses 0.1 BPM. A fast turn can use a 1 BPM step.
Keep the existing A1 creation and Clear A1 confirmation rules.
On Files, RIGHT acts only if the selected item is a folder. The existing Load
and Take actions handle files. Use the existing bounds for lists and edit pages.
The K1 pad move stops at the row edge.

| Page | K2 | K3 | K4 | K5 | K6 | K7 |
| --- | --- | --- | --- | --- | --- | --- |
| Pads or Pad edit TONE | Mute group | Volume | Pan | Pitch | Fine tune | — |
| Pads or Pad edit ENV | — | Attack | Decay | Hold | Release | — |
| Pads or Pad edit FILTER | Type | Cutoff | Resonance | Envelope amount | Envelope decay | Key tracking |
| Pads or Pad edit SOURCE | Source | Pad region start | Pad region end | Loop start | Loop end | — |
| Pads or Pad edit FX | Drive | Send FX 1 | Send FX 2 | — | — | — |
| Pattern transport | Tempo | Quantize | Strength | Bars | Swing | — |
| Chop and Chop edit | Zoom | Slice start | Slice end | Chop region start | Chop region end | Threshold |
| Settings | Selected value | — | — | — | — | — |
| FX | Type | Slot value A | Slot value B | — | — | — |

A dash is an empty position. Dim the values that the sampler does not offer yet.
The Pads inspector and edit use the same page names and positions.
For the two FX slots, show the current effect names for value A and value B.
Follow the keyboard value bounds and apply the same Undo and safety rules.

For Chop, the K3 and K4 pair changes the selected slice. The K5 and K6 pair
changes the whole Chop region. Reuse the current relative bound steps and link
updates. On K2, each isolated step applies one quarter of a keyboard zoom step
on the log scale. The current frame limits and rounding still apply.
Apply a reverse turn at once. Store no pending zoom movement.
Retain the relative acceleration rule for supported values and fast zoom turns.
Keep the one dB threshold comparison for slow turns. Move that role to K7.

## Bank 2

| Row | Left | Right |
| --- | --- | --- |
| 7–8 | Pan (dim until pad-sound) | Master |
| 5–6 | Send FX 1 | Send FX 2 |
| 3–4 | Pitch | Decay |
| 1–2 | Volume | Cutoff |

Use the selected pad on each screen. On Chop and Files, sample selection does
not change the pad target. Use the selected pad from the sampler state.
Keep the roles while the command panel shows. Follow existing overlay priorities.
Both master controls use the same action.

## Panel, tempo and deferred roles

Keep Cut and the existing region and marker actions on their panel routes.
Remove a value, parameter or editor page cell when a knob or button replaces it.
Show unused cells dim. Keep the keyboard and learn actions.
Mark and test the order of symmetric pairs in the page tables and panel map.
Use the physical grid width when checking pairs. A pair cannot cross a row.

Use tempo on the Pattern transport page. Remove the shipped global knob and
panel tempo steps. Keep tempo keyboard and MIDI learn actions.
Keep a dim Tap Tempo position until the action exists. When it exists, use the
`tempo` cell on the Pattern context route. Pan follows `pad-sound`.
When change 100 adds ptsr Note Repeat, use RATE, SWING, TEMPO and LATCH on bank 1
knobs 1–4 while it runs. The matching labels are on the device. This change does
not add Note Repeat, swing or the other missing sound parameters.

From the visible main page, More and the action cost two presses on page 1.
Each later context page adds one Next press. The PAD BANK switch and a missed
reveal add separate presses. See [the count table](../../../docs/CONTROLLER-PATHS.md).
Keep frequent actions on a direct route or the first context page.

## Existing saved assignments

`App::seed_device` copies shipped roles to saved bindings on first connection.
It does not store the origin of each entry. A reset discards the whole saved map.
Do not infer origin by equality with the old factory action.
Use the new layout for a new profile. For an existing profile, offer a review
before replacing saved roles. Show the prior and proposed role per affected slot.
Keep a personal or disabled binding. Ask the owner about an ambiguous old role.
Save only the accepted result. Cancel leaves the map as it was.
If persistence fails, keep the old effective map. A retry must not repeat a
partially applied update. Other device profiles and USB preset records keep their roles.

## Knob strip and help

Show two 2 by 4 grids side by side, top row 7–8 to bottom row 1–2.
Mark the last bank that sent valid knob movement on this connection.
Clear the mark on disconnect. Dim any inactive role. Show the effective page
and list name in the strip and the help summary row. Check 80 by 26 and 120 by 40.
The mark does not promise the physical switch position after a bank switch.

## Edge cases before implementation

| Case | Expected result | Status |
| --- | --- | --- |
| A new bank sends a turn before the next redraw. | Use the CC bank immediately. Show that bank on the next frame. | Support now with raw MIDI. |
| A question follows a knob or button event. | P7 and P8 answer once. No knob changes a value behind the question. | Preserve with input-order tests. |
| A held Gate crosses a bank or page change. | Its release reaches the original voice. | Preserve with raw MIDI. |
| A turn follows a stale disconnect event from the prior connection. | The old event cannot mark or change the new connection. | Preserve connection ownership. |
| One pair lies across a row boundary or the data reverses its roles. | The layout data check fails. | Support now with table tests. |
| A slow zoom turn reverses at a limit. | The reverse turn works without a jump. | Support now with controlled turn timing. |
| A saved factory value and an owner value look the same. | The owner sees the proposed replacement before approval. | Support now with persistence checks. |
| Another controller sends a turn during an SMC-PAD mode change. | Its current input route and hold stay active. | Preserve with two-device replay. |
| PAD BANK target flow needs a new count. | Measure target flows on the device before the later change. | Later panel change. |

## Open questions

- The first release can show dim controls for unavailable parameters. The
  default is to keep their assigned physical places for later changes.
- The layout update must expose its exact list of conflicting assignments.
  The default is to retain each old assignment until the owner resolves it.
- Target flow counts require the physical device. The later panel change
  will compare both routes before it selects one.

## Requirements for later implementation steps
The current spec delta covers the smaller Chop zoom step.
Add each section below to its spec delta before its implementation step.
Each active scenario needs a named test.

### Controller reach

#### ADDED Requirements

##### Requirement: The list buttons follow the current SMC-PAD mode

The shipped LEFT and RIGHT buttons SHALL move the main list of the current mode.
They SHALL select pad banks on Pads, edit pages in pad edit and bar pages in Pattern.
They SHALL select slices on Chop, tabs in Settings and slots in FX.
On Files, LEFT SHALL select the parent folder. RIGHT SHALL enter the selected folder.
A selected file SHALL keep its current Load and Take paths.
An active picker or input overlay SHALL retain its input priority.
A held repeat SHALL follow the list bounds.

###### Scenario: SMC-PAD list buttons name and move the same list
- **WHEN** the shipped buttons act on Pads, pad edit, Pattern, Chop, FX or Settings
- **THEN** each button moves its named list in the direction shown
- **AND** a turn or a hold cannot bypass the list bounds

###### Scenario: File list buttons navigate folders
- **WHEN** the owner turns bank 1 knob 1 to a file and presses RIGHT
- **THEN** the preview target and pad assignment stay the same
- **AND** RIGHT enters the folder after knob 1 selects it, and LEFT returns to its parent

##### Requirement: Bank 1 provides position and page values

Bank 1 knob 1 SHALL move the position. It SHALL scroll projects only in the picker.
Knobs 2–7 SHALL change the page values in the design.
Pads and pad edit SHALL use the current inspector page positions.
Unavailable values SHALL stay dim and inactive. Knob 8 SHALL change master volume.
Current input owners SHALL keep their priority.

###### Scenario: Pad inspector and editor share knob positions
- **WHEN** the owner selects each Pad page and turns its assigned bank 1 knobs
- **THEN** the same knobs change the same values in the inspector and editor
- **AND** an unavailable role changes no value and a different screen uses its own page

###### Scenario: Knob 1 moves the current position
- **WHEN** the owner turns bank 1 knob 1 on Pattern, Files, Settings, FX or a picker
- **THEN** the turn moves the current step, item, parameter or row for that mode
- **AND** project selection changes only while its picker owns input

###### Scenario: Chop knobs move two separate bound pairs
- **WHEN** the owner turns bank 1 knobs 3–6 on Chop or Chop edit
- **THEN** each turn changes only the named slice or region edge as the keyboard path does
- **AND** it keeps the current limit, link and Undo rules

##### Requirement: Bank 2 follows the selected pad on each screen

The shipped bank 2 roles SHALL be volume, cutoff, pitch, decay, send 1 and send 2 on knobs 1–6.
Knob 7 SHALL reserve the pan role. It SHALL stay dim until pan is available.
Knob 8 SHALL change master volume through the same action as bank 1 knob 8.
The effect sends SHALL use the selected pad and the existing send limits.
The bank 2 roles SHALL retain their meaning through a screen change.
The program SHALL keep owner bindings and current input overlay rules.

###### Scenario: Bank 2 supplies the same live pad controls on every screen
- **WHEN** the owner selects a pad and turns bank 2 knobs 1–7 across screens and panel pages
- **THEN** available roles change the named value of that pad
- **AND** send 1 and send 2 change different slots and the dim pan role has no action

###### Scenario: Both banks pin master to the top right knob
- **WHEN** the owner turns physical knob 8 in bank 1 and then in bank 2
- **THEN** both turns change the same master volume
- **AND** neither turn changes a bank, pad parameter or tempo

##### Requirement: The shipped SMC-PAD tempo knob is local to Pattern transport

Pattern transport SHALL provide tempo, Bars, Quantize and Strength. Swing SHALL stay dim.
A slow tempo step SHALL change 0.1 BPM. A fast turn SHALL support 1 BPM steps.
The shipped map SHALL restrict tempo knobs to Pattern transport.
The Pattern context route SHALL reserve a dim Tap Tempo cell.
Keyboard and learned tempo actions SHALL keep their routes.

###### Scenario: Tempo turns act only on the Pattern transport page
- **WHEN** the owner turns the tempo position on the Pattern transport page
- **THEN** the current tempo handler changes the same value as its keyboard path
- **AND** that knob on Chop, Files, Settings or another page cannot change tempo

##### Requirement: Panel buttons select context pages

The shipped More, Next and Main cells SHALL move between panel pages.
A bank 1 knob turn SHALL keep its screen role while the panel shows.
A panel page change SHALL update the visible cell labels before a later cell acts.
More and one action cell SHALL be sufficient for a first context page action.

###### Scenario: Next selects a later context page
- **WHEN** the panel shows a context page and the owner turns bank 1 knob 6
- **THEN** the turn keeps its screen role and the panel page stays the same
- **AND** Next selects the later context page before a cell acts once

#### MODIFIED Requirements

##### Requirement: Current actions have paths from both inputs

Each current non-text keyboard action SHALL have a production route from the shipped SMC-PAD map.
Tempo may use only Pattern transport. Pad bank changes may use the Pads screen list.
Each shipped controller action SHALL have a keyboard route with the same result.
Chop slice bounds SHALL select the same bound as their keyboard routes.

###### Scenario: Every current keyboard action has a controller route
- **WHEN** the reach test visits each screen and mode with the shipped maps
- **THEN** it finds the controller route for each current non-text action, including the local tempo route
- **AND** the test reports the screen, mode and action if it finds no route

###### Scenario: Every current controller action has a keyboard route
- **WHEN** the reach test visits each effective control in the shipped MIDI map
- **THEN** each sampler action has a keyboard route with the same result
- **AND** either route to a slice bound selects the same bound

##### Requirement: A1 controls use the shipped pattern encoder context

While A1 controls are open, the shipped Pattern roles SHALL take priority over underlying screen roles.
Bank 1 knob 1 SHALL select the parameter. The page knobs SHALL change their named values.
The existing keyboard routes SHALL keep their bounds and results.
Closing A1 controls SHALL restore the underlying screen roles.
Other devices, master volume, panel cells and held releases SHALL keep their routes.
Input overlays SHALL keep priority. The Controller tab SHALL show the current roles.

###### Scenario: A1 encoders follow the open controls on every screen
- **WHEN** separate keyboard and raw MIDI runs open A1 controls and change Bars, Quantize or Strength
- **THEN** both runs reach the same settings and bounds
- **AND** knob 1 selects the current parameter and the visible map names its role
- **AND** closing A1 controls restores the underlying roles

##### Requirement: A1 encoder bindings survive profile seeding

The Pattern roles SHALL have a separate learned group.
The owner's Pattern bindings SHALL take priority over the shipped roles.
A new SMC-PAD profile SHALL seed the group.
An existing profile SHALL follow the saved layout update rules, including for an empty group.
Learning or removing a binding SHALL retain the effective result after restart.

###### Scenario: A1 encoder bindings survive profile seeding and owner changes
- **WHEN** a new or existing SMC-PAD profile loads and the owner replaces or removes a Pattern binding
- **THEN** the saved choice takes priority while A1 controls are open
- **AND** unrelated groups and other devices keep their assignments

###### Scenario: A1 encoder context preserves input owners and held pads
- **WHEN** an input overlay opens over A1 controls or a held Gate crosses their close
- **THEN** the active input owner keeps priority and no A1 setting changes behind it
- **AND** the Gate release stops its original pad

##### Requirement: Chop threshold has matching keyboard steps

On Chop and Chop edit, Ctrl+Alt+Up SHALL increase the hit threshold by 1 dB.
Ctrl+Alt+Down SHALL decrease it by 1 dB.
The keys SHALL keep their existing threshold actions.
An isolated bank 1 knob 7 step SHALL use the same threshold interval.
Fast turns SHALL keep their current acceleration.
The threshold SHALL stay within -40 to 0 dB.

###### Scenario: Keyboard and encoder set the same Chop threshold
- **WHEN** keyboard and raw MIDI runs step the threshold in both directions at least 100 ms apart
- **THEN** each run reaches the same threshold, cuts, linked pads and Undo state
- **AND** an attempt to pass a bound creates no Undo entry

###### Scenario: Fast encoder turns keep acceleration
- **WHEN** bank 1 knob 7 sends two movement steps within 40 ms on Chop
- **THEN** the existing MIDI speed rule accelerates the second step
- **AND** two separate keyboard presses still change the threshold by exactly 2 dB

###### Scenario: Threshold keys respect owner bindings and input owners
- **WHEN** an owner replaces or removes a threshold key, or an input overlay owns input
- **THEN** the owner entry or active mode keeps priority

##### Requirement: Pads have a silent keyboard selection route

On Pads and pad edit, Alt+Left and Alt+Right SHALL select the adjacent pad in the same row.
On Pads, the shipped bank 1 knob 1 SHALL use this silent route.
Selection SHALL stop at a row edge. It SHALL preserve the target of each held pad.
A key repeat SHALL not select another pad. Other screens SHALL retain their Alt-arrow routes.
An owner binding or active overlay SHALL keep priority.

###### Scenario: Keyboard selects a neighboring pad without sound
- **WHEN** the owner uses Alt+Left or Alt+Right on Pads or pad edit
- **THEN** the selected pad matches the corresponding bank 1 knob 1 move on Pads
- **AND** the move stops at a row edge without pad sound, a changed value or an ended splash
- **AND** a key repeat does not move again and a held Gate keeps its original release

###### Scenario: Silent selection respects owner bindings and overlays
- **WHEN** an owner changes an Alt-arrow key or an overlay owns it
- **THEN** the owner entry or overlay acts first and the selected pad stays unchanged
- **AND** Chop keeps its current Alt-arrow slice-bound route

##### Requirement: Controller routes keep short press paths

Frequent actions SHALL have a direct button, pad, knob or first context page route.
The route table SHALL list the route and counts for each User Story.
From the visible main page, the first context page SHALL take two presses.
Each later context page SHALL add one Next press.
From a hidden panel, count PAD BANK and a reveal hit when the shortcut misses.
Count knob turns, target pad hits and modal answers separately.

###### Scenario: First entry and repeated commands have separate counts
- **WHEN** the owner enters a context page from a hidden or visible panel
- **THEN** only the hidden count includes PAD BANK and a reveal hit when needed
- **AND** each Next press counts once and a knob turn counts as zero presses

###### Scenario: Every story has a recorded controller path
- **WHEN** the reach test reads each action story in User Stories
- **THEN** the route table gives an SMC-PAD path and count
- **AND** observation-only, text-only and other-device stories name their exception

##### Requirement: Added context cells provide missing actions

Each view SHALL keep its main page, first context page and fixed bottom row.
The common and view-specific cells below SHALL keep their existing actions.
Current keyboard and MIDI learn actions SHALL remain available.

| View | Page | Cells in order |
| --- | --- | --- |
| Every view | 2 | Output record, transport, take, project picker, save, pad grid, learn, A1 undo, A1 controls, exit |
| Every view | 3 | `pattern.erase`, pattern record |
| Pads | 4 | Previous grid view, next grid view, empty rows, step edit |
| Chop and Chop edit | 4 | Hits back, `marker.delete`, `slice.end_delete`, assign order |

###### Scenario: Added cells reach their current actions
- **WHEN** the owner uses a listed cell where its action is available
- **THEN** the action has the same result as its keyboard path
- **AND** another mode does not show it as available if it has no effect there

#### REMOVED Requirements

##### Requirement: Context page selection preserves the screen controls

**Reason**: Knob 6 now changes a page value. Discrete page moves use cells.
**Migration**: Use More, Next and Main as specified in "Panel buttons select context pages".

### Command panel

#### ADDED Requirements

##### Requirement: Shipped control pairs share a physical row

The shipped knob page tables and panel map SHALL identify both roles in a symmetric pair.
Each pair SHALL use adjacent cells in one physical row.
The lower, earlier or negative role SHALL be on the left.
The layout check SHALL cover slice bounds, region bounds, FX sends and other named pairs.
It SHALL cover No and Yes on P7 and P8, and Undo and Redo on the bottom row.

###### Scenario: Layout rejects a split or reversed pair
- **WHEN** a test puts one member of a shipped pair on another row or reverses the roles
- **THEN** the layout check reports the group and both positions

#### MODIFIED Requirements

##### Requirement: Panel pages expose screen and context commands

The main page SHALL offer Pads, Files, Chop, Pattern, FX, Settings, Help and More.
It SHALL provide up to four current-screen actions. Unused cells SHALL stay dim.
More SHALL open the first context page. Each context page SHALL have up to ten actions.
It SHALL keep Next in cell 11 and Main in cell 12.
Next SHALL cycle through context pages. The program SHALL show the page and count.
A screen change or panel exit SHALL select the main page again.
LEFT and RIGHT SHALL retain the current screen or mode list.
The shipped page SHALL remove value, parameter and edit-page cells that direct controls replace.
The program SHALL keep their keyboard and MIDI learn actions.

###### Scenario: Screen and context pages cover more than 16 commands
- **WHEN** the owner selects More, cycles through context pages and presses Main
- **THEN** each page shows its commands in the physical cell order
- **AND** the page move retains the fixed bottom row, count and transport

###### Scenario: A screen switch resets the panel page
- **WHEN** a visible context page is open and the owner changes screen
- **THEN** the new screen shows its main page and current cells
- **AND** LEFT and RIGHT move the current screen list

###### Scenario: Main cells follow the current view
- **WHEN** the owner visits Pads, pad edit, Files, Chop, Pattern, FX and Settings
- **THEN** cells 9–12 show and run the available actions for that view
- **AND** unused cells stay dim and change no value

###### Scenario: Direct page controls replace panel steps
- **WHEN** the owner opens an editor and changes its page, parameter and value
- **THEN** the buttons and knobs provide the direct routes
- **AND** the shipped panel omits the replaced steps
- **AND** the keyboard and learn actions remain available

### Input devices

#### ADDED Requirements

##### Requirement: An existing SMC-PAD map can adopt the reviewed layout

New SMC-PAD profiles SHALL use the shipped roles.
An existing map SHALL show the old and proposed roles before it changes.
The program SHALL keep owner bindings and removals.
An old role with unknown origin SHALL require approval before replacement.
Only approved changes SHALL persist. Cancel or save failure SHALL keep the old map.
Other device profiles and USB preset records SHALL keep their state.

###### Scenario: Saved SMC-PAD roles survive the layout review
- **WHEN** the owner reviews a saved map with a personal, removed or ambiguous assignment
- **THEN** the review shows which roles can change
- **AND** the program keeps personal and removed assignments
- **AND** only the approved changes persist after restart

###### Scenario: Canceled or refused layout updates are safe to retry
- **WHEN** the owner cancels the review or the program cannot save the approved map
- **THEN** the old bindings still act and the other device keeps its own bindings
- **AND** a later successful retry applies the approved changes once

### Screen layout

#### ADDED Requirements

##### Requirement: The SMC-PAD knob strip shows both banks and the current list

The knob strip SHALL show both banks side by side at 80 by 26 and 120 by 40.
Each bank SHALL show its physical rows in this order from top to bottom: 7–8, 5–6, 3–4, 1–2.
The strip SHALL name the page and dim unavailable roles.
The strip and one-row help SHALL name the LEFT and RIGHT roles.
The strip SHALL mark the bank of the last valid turn from its connection.
A disconnect SHALL clear the last input mark.
The F1 Controller tab SHALL show the same effective roles.

###### Scenario: The strip identifies page, list and roles at both sizes
- **WHEN** the owner views each supported size with the SMC-PAD connected
- **THEN** both knob banks fit in physical order
- **AND** the strip and help name the current page or list and unavailable roles stay dim

###### Scenario: Last knob input marks its bank
- **WHEN** valid knob movement arrives from bank 1 and then bank 2
- **THEN** the mark changes to the bank that sent the last turn
- **AND** no mark changes while a bank switch sends no message
- **AND** a disconnect clears the mark and stale input cannot set it again
