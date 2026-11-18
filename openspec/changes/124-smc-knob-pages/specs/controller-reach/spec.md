## ADDED Requirements

### Requirement: SMC-PAD Chop zoom uses smaller relative steps

On Chop and Chop edit, bank 1 knob 2 SHALL change zoom in quarter steps on the log scale.
The turn SHALL keep the cursor anchor and current limits.
The knob SHALL apply movement immediately and retain acceleration.
Keyboard and other device zoom routes SHALL keep their step sizes.
Assign and modal input rules SHALL keep their priority.

#### Scenario: Slow Chop zoom uses smaller input steps
- **WHEN** the owner sends four isolated movement steps on bank 1 knob 2
- **THEN** each step changes the zoom by a quarter of a keyboard step on the log scale
- **AND** the four steps give one keyboard step within frame rounding
- **AND** reverse turns act at once and fast turns retain acceleration

#### Scenario: SMC-PAD Chop zoom preserves Assign and question priority
- **WHEN** the owner turns bank 1 knob 2 during Assign or a question
- **THEN** the current view stays in place and the mode keeps control

### Requirement: Chop knobs provide separate slice and region pairs

On Chop and Chop edit, bank 1 knobs 3 and 4 SHALL change the slice start and end.
Knobs 5 and 6 SHALL change the Chop region start and end.
The pairs SHALL keep the current limits, links and Undo rules.
Assign and visible questions SHALL block bound turns.

#### Scenario: Chop knobs move two separate bound pairs
- **WHEN** the owner turns bank 1 knobs 3–6 on Chop or Chop edit
- **THEN** each turn moves only the named edge
- **AND** Undo restores the change with its affected pad links

#### Scenario: Chop bounds wait during Assign and questions
- **WHEN** the owner turns a bound knob during Assign or a visible question
- **THEN** the bounds, plan and question stay in place

### Requirement: Bank 2 follows the selected pad on each screen

The shipped bank 2 knobs 1–6 SHALL be volume, cutoff, pitch, decay, send 1 and send 2.
Each turn SHALL target the selected pad, even if other pads have marks.
Knob 7 SHALL stay inactive until pan is available.
Knob 8 SHALL adjust master volume.
The bank 2 roles SHALL keep their meaning across screens and panel pages.
The program SHALL keep owner bindings and input overlay rules.

#### Scenario: Bank 2 supplies the same live pad controls on every screen
- **WHEN** the owner selects a pad and turns bank 2 knobs 1–7 across screens and panel pages
- **THEN** available roles change the named value of that pad
- **AND** send 1 and send 2 change different slots and the dim pan role has no action
- **AND** marked pads stay unchanged

### Requirement: The list buttons follow the current SMC-PAD mode

LEFT and RIGHT SHALL select pad banks, pad edit pages, Pattern bars, Chop slices,
Settings tabs or FX slots in the corresponding view.
On Files, LEFT SHALL select the parent folder. RIGHT SHALL enter the selected folder.
The current Load and Take paths SHALL handle files.
The input overlay SHALL keep its priority.

#### Scenario: SMC-PAD list buttons name and move the same list
- **WHEN** LEFT and RIGHT act on Pads, pad edit, Pattern, Chop, FX or Settings
- **THEN** each button moves the list for that mode
- **AND** the current selection remains within the list bounds

#### Scenario: File list buttons navigate folders
- **WHEN** the owner uses RIGHT with a selected file or folder
- **THEN** a file remains selected and a folder opens
- **AND** LEFT returns to the parent folder

### Requirement: Both SMC-PAD banks pin master to knob 8

Both bank 1 and bank 2 knob 8 SHALL change master volume through the same action.
The shipped knob SHALL keep its role through screen and panel changes.

#### Scenario: Both banks pin master to the top right knob
- **WHEN** the owner turns physical knob 8 in bank 1 and then in bank 2
- **THEN** both turns change master volume
- **AND** neither turn changes a bank, pad parameter or tempo

## MODIFIED Requirements

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

### Requirement: Context page selection preserves the screen controls

On Chop and Chop edit, first-bank knob 6 SHALL keep its region-end role.
On other screens, knob 6 SHALL select a context page only while the panel shows.
The program SHALL show the selected page and its labels before another cell acts.
More and one context cell SHALL reach a first context page action.
On Chop and Chop edit, the Next cell SHALL move to later context pages.

#### Scenario: Encoder selects a later context page
- **WHEN** the panel shows a context page outside Chop and Chop edit and the owner turns knob 6
- **THEN** the later page labels show before another cell acts
- **AND** the screen value and bank stay in place

#### Scenario: A Chop bound turn keeps the current panel page
- **WHEN** the owner turns knob 6 with the Chop context page visible
- **THEN** the turn changes the region end and keeps the panel page
- **AND** the Next cell moves to a later context page

### Requirement: Current actions have paths from both inputs

For each current screen and mode, each effective keyboard action that needs no typed text SHALL have a route from the shipped SMC-PAD profile. Pad bank changes MAY use the Pads screen list. Each shipped SMC-PAD sampler action SHALL have a keyboard key or key path with the same result. A Chop slice-bound turn SHALL select the same start or end bound as its keyboard route. The route SHALL use production input dispatch.

#### Scenario: Every current keyboard action has a controller route
- **WHEN** the reach test visits each current screen and mode with the shipped keyboard map, SMC-PAD profile and panel map
- **THEN** each non-text keyboard action reaches the same result through raw SMC-PAD MIDI
- **AND** the test reports the screen, mode and action if no controller route exists

#### Scenario: Every current controller action has a keyboard route
- **WHEN** the reach test visits each shipped SMC-PAD button, pad, panel cell and knob role
- **THEN** each sampler action reaches the same result from the keyboard
- **AND** a Chop slice-bound turn selects the same start or end bound as its keyboard route
- **AND** the test reports any action without a key or key path, apart from panel navigation

### Requirement: Pads have a silent keyboard selection route

On Pads and Pads edit, `Alt+Left` and `Alt+Right` SHALL select the adjacent pad in the same row, as SMC-PAD bank 1 knob 1 does on Pads. Selection at a row edge SHALL stay in place. These keys SHALL not hit a pad, change its value, stop a held Gate pad or end the splash. A key repeat SHALL not select another pad. Other screens SHALL keep their current Alt-arrow routes. Owner keymap entries and an active text line, question, picker, Help or learning mode SHALL keep priority.

#### Scenario: Keyboard selects a neighboring pad without sound
- **WHEN** the owner presses `Alt+Left` or `Alt+Right` on Pads or Pads edit
- **THEN** the selected pad matches the corresponding bank 1 knob 1 turn on Pads
- **AND** no pad plays, a held Gate keeps its release target and a repeat does not move again
- **AND** selection stops at the row edge without changing a value or ending the splash

#### Scenario: Silent selection respects owner bindings and overlays
- **WHEN** an owner keymap replaces an Alt-arrow key or sets it to `none`, or an input overlay owns the key
- **THEN** the owner entry or overlay acts first and the pad selection stays unchanged
- **AND** Chop keeps its existing Alt-arrow slice-bound route

### Requirement: Added context cells provide missing actions

Every panel view SHALL keep its main page, first context page and fixed bottom row. It SHALL add the common action cells in the table. Pads, Chop and Chop edit SHALL add the listed page 4 cells. Outside Chop and Chop edit, encoder 7 SHALL keep its tempo role until the Pattern transport page is available.

#### Scenario: Added cells reach their current actions
- **WHEN** the owner uses a listed cell in a screen or mode where its action works
- **THEN** the action has the same result as its keyboard path
- **AND** another screen or mode does not claim a route for a command that has no effect there

| View | Page | Cells in order: label (`action`) |
| --- | --- | --- |
| Every view | 2 | `outrec` (`record.output`), `trnspt` (`transport.toggle`), `take` (`pad.take`), `proj` (`project.pick`), `save` (`session.save`), `grid` (`ui.pads_toggle`), `learn` (`learn.toggle`), `A1undo` (`pattern.undo`), `A1 ctl` (`pattern.controls`), `exit` (`app.quit`) |
| Every view | 3 | `bpm -` (`transport.tempo_down`), `bpm +` (`transport.tempo_up`), `bpm-.1` (`transport.tempo_down_fine`), `bpm+.1` (`transport.tempo_up_fine`), `erase` (`pattern.erase`), `A1 rec` (`pattern.record`) |
| Pads | 4 | `view -` (`pattern.view_prev`), `view +` (`pattern.view_next`), `rows` (`pattern.overview_empty_toggle`), `step` (`pattern.step_edit`) |
| Chop, Chop edit | 4 | `hits -` (`chop.hits_back`), `delcut` (`marker.delete`), `delend` (`slice.end_delete`), `order` (`chop.assign_order`) |
