# controller-reach Specification

## ADDED Requirements

### Requirement: Current actions have paths from both inputs

For each current screen and mode, each effective keyboard action that needs no typed text SHALL have a route from the shipped SMC-PAD profile. Each shipped SMC-PAD sampler action SHALL have a keyboard key or key path with the same result. The route SHALL use production input dispatch.

#### Scenario: Every current keyboard action has a controller route
- **WHEN** the reach test visits each current screen and mode with the shipped keyboard map, SMC-PAD profile and panel map
- **THEN** each non-text keyboard action reaches the same result through raw SMC-PAD MIDI
- **AND** the test reports the screen, mode and action if no controller route exists

#### Scenario: Every current controller action has a keyboard route
- **WHEN** the reach test visits each shipped SMC-PAD button, pad, panel cell and knob role
- **THEN** each sampler action reaches the same result from the keyboard
- **AND** the test reports any action without a key or key path, apart from panel navigation

### Requirement: Chop threshold has matching keyboard steps

On Chop and Chop edit, `Ctrl+Alt+Up` SHALL increase the hit threshold by 1 dB and `Ctrl+Alt+Down` SHALL decrease it by 1 dB. These keys SHALL use `chop.threshold_up` and `chop.threshold_down`. Each step SHALL have the same result as one isolated opposite-direction detent of SMC-PAD encoder 3. Fast MIDI turns SHALL keep their existing acceleration. The threshold SHALL stay within -40 to 0 dB.

#### Scenario: Keyboard and encoder set the same Chop threshold
- **WHEN** keyboard and raw SMC-PAD runs step the Chop threshold in both directions and modes, with encoder detents at least 100 ms apart
- **THEN** each run reaches the same threshold, cuts, linked-pad result and Undo state
- **AND** neither input moves beyond -40 to 0 dB or adds an Undo entry at a bound

#### Scenario: Fast encoder turns keep acceleration
- **WHEN** the SMC-PAD sends two encoder 3 detents within 40 ms on Chop
- **THEN** the existing MIDI speed rule accelerates the second threshold step
- **AND** two separate keyboard presses still change the threshold by exactly 2 dB

#### Scenario: Threshold keys respect owner bindings and input owners
- **WHEN** an owner map replaces a threshold key or sets it to `none`, or assign mode or an input overlay owns input
- **THEN** the owner entry or active mode keeps priority and the threshold stays unchanged

### Requirement: Pads have a silent keyboard selection route

On Pads and Pads edit, `Alt+Left` and `Alt+Right` SHALL select the adjacent pad in the same row, as SMC-PAD LEFT and RIGHT do. Selection at a row edge SHALL stay in place. These keys SHALL not hit a pad, change its value, stop a held Gate pad or end the splash. A key repeat SHALL not select another pad. Other screens SHALL keep their current Alt-arrow routes. Owner keymap entries and an active text line, question, picker, Help or learning mode SHALL keep priority.

#### Scenario: Keyboard selects a neighboring pad without sound
- **WHEN** the owner presses `Alt+Left` or `Alt+Right` on Pads or Pads edit
- **THEN** the selected pad matches the result of SMC-PAD LEFT or RIGHT
- **AND** no pad plays, a held Gate keeps its release target and a repeat does not move again
- **AND** selection stops at the row edge without changing a value or ending the splash

#### Scenario: Silent selection respects owner bindings and overlays
- **WHEN** an owner keymap replaces an Alt-arrow key or sets it to `none`, or an input overlay owns the key
- **THEN** the owner entry or overlay acts first and the pad selection stays unchanged
- **AND** Chop keeps its existing Alt-arrow slice-bound route

### Requirement: Input exceptions stay explicit

The only controller-only steps SHALL be PAD BANK selection, its status mark and page navigation within its panel. The only keyboard-only operations SHALL be `pad.rename`, `browser.filter`, a typed new-project name and keymap edits. `app.quit` SHALL remain unavailable for MIDI learn. No other exception SHALL enter the inventory without an owner decision.

#### Scenario: Text entry keeps its exceptions
- **WHEN** the reach inventory examines pad naming, file filtering, a typed new-project name, keymap edits and quit
- **THEN** it marks only the four text operations as keyboard-only
- **AND** MIDI learn does not offer `app.quit`

### Requirement: Controller routes keep short press paths

A current frequent action SHALL have a direct SMC-PAD button, pad or knob route. Each other non-text action SHALL need at most two presses from the visible main panel. Knob turns, target pad hits and later modal answers do not count toward this limit. The reach table SHALL state the count from a hidden panel separately. That count SHALL include PAD BANK selection, a reveal hit and the action hit. It SHALL name a route and both counts for every action story in `docs/USER-STORIES.md`.

#### Scenario: First entry and repeated commands have separate counts
- **WHEN** the owner reaches a context command from a hidden panel and then repeats it from a visible panel
- **THEN** the first count includes PAD BANK and the reveal hit, and the later count does not
- **AND** neither count treats a knob turn as a button press

#### Scenario: Every story has a recorded controller path
- **WHEN** the reach test reads every action story in `docs/USER-STORIES.md`
- **THEN** each story has an SMC-PAD route and a press count in `docs/CONTROLLER-PATHS.md`
- **AND** observation-only, text-only and other-device stories name their exception

### Requirement: Context page selection preserves the screen controls

While the SMC-PAD panel is visible, first-bank encoder 6 SHALL select a context page without changing the current screen value or bank. The panel SHALL show the selected page and its current cell labels before another cell acts. More and one context cell SHALL remain sufficient from the visible main panel. While the panel is hidden, encoder 6 SHALL remain unassigned.

#### Scenario: Encoder selects a later context page
- **WHEN** the panel shows its main page and the owner hits More, turns encoder 6 to a later context page and hits a cell
- **THEN** the later page labels show before the cell runs its action once
- **AND** LEFT and RIGHT still select items, while Next, Main, fixed bottom cells, owner overrides, transport and sounding pads keep their behavior

### Requirement: Added context cells provide missing actions

Every panel view SHALL keep its main page, first context page and fixed bottom row. It SHALL add the common action cells in the table. Pads, Chop and Chop edit SHALL add the listed page 4 cells. First-bank encoder 7 SHALL step tempo, and encoder 8 SHALL select banks.

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

### Requirement: Pickers accept SMC-PAD selection without typed text

In the project picker, SMC-PAD encoder 1 SHALL move the selection. P8 SHALL confirm the selected project, and P7 or Back SHALL cancel. These modal routes SHALL take priority over the screen behind the picker. A visible question SHALL keep its existing P7 and P8 priority. A failed project transition SHALL keep the current project, picker and selection.

#### Scenario: Controller opens a selected project
- **WHEN** the owner opens the project picker, turns encoder 1 and presses P8
- **THEN** the selected project opens after its required save succeeds
- **AND** a refused save keeps the old project and the picker selection

### Requirement: Paths folder picker accepts SMC-PAD selection

In the Paths folder picker, SMC-PAD encoder 1 SHALL move the folder selection. LEFT and RIGHT SHALL move to parent and child folders. P8 SHALL select the current folder, and P7 or Back SHALL cancel. These routes SHALL not change the Settings value behind the picker.

#### Scenario: Controller selects a Paths folder
- **WHEN** the owner opens the Paths folder picker, turns encoder 1, moves into a folder with RIGHT and presses P8
- **THEN** only the selected folder becomes the path
- **AND** P7 in a separate run cancels without changing the path

### Requirement: Settings values have a controller route

On Settings, SMC-PAD encoder 2 SHALL change the selected value with the same bounds and result as the keyboard value keys. While a picker is open, its controls SHALL not change a Settings value.

#### Scenario: Settings knob changes only the selected value
- **WHEN** separate keyboard and raw SMC-PAD runs select the same Settings row and change its value once
- **THEN** both runs reach the same value and bounds
- **AND** a knob turn during a Paths folder picker changes only its selection

### Requirement: Help has a controller route

The main panel Help cell SHALL open Help. While Help shows, P7 and P8 SHALL select the previous and next Help tab. P9 and P10 SHALL scroll Help or Messages, or move the selected control on the Controller tab. Back SHALL close Help. Help SHALL show these controller controls. A Help cell SHALL not act on the screen behind Help. Keyboard Help keys and their results SHALL stay available.

#### Scenario: Help cells navigate without acting behind Help
- **WHEN** the owner opens Help from the panel, uses P7 through P10 and then Back
- **THEN** only the Help tab, scroll position or selected controller control changes before Help closes
- **AND** the screen and its selected value remain unchanged

### Requirement: Controller can finish learning

With an F10 or MIDI-tab learning choice pending, P8 SHALL save that choice, P7 SHALL cancel it and Back SHALL leave learning. The answer SHALL not replace the chosen control or run a panel action. With no choice pending, P7 and P8 SHALL remain available as controls to learn.

#### Scenario: Learning answer keeps its chosen control
- **WHEN** the owner selects an action, presses P7 or P8 as the control to learn and then presses P8 again
- **THEN** the first press becomes the pending control and the second saves it once
- **AND** a separate pending choice followed by P7 cancels without a new choice or screen action

### Requirement: SMC-PAD Exit asks before quit

An explicit Exit context cell SHALL open a visible question without starting quit. P8 SHALL confirm and enter the existing exit-save path once. P7 or Back SHALL cancel and keep the session open. Other panel cells SHALL NOT act behind the question. Ctrl+Q SHALL keep its current direct exit behavior. MIDI learn SHALL NOT offer Exit.

#### Scenario: Exit needs a second SMC-PAD press
- **WHEN** the owner hits Exit on the panel and presses P7
- **THEN** the question closes and the session remains open
- **AND** a later Exit followed by P8 enters the existing exit-save path once

### Requirement: Reach tests preserve modal safety and visible modes

The reach test SHALL use the production question, picker and learning routes. It SHALL test that each current mode shows its badge and that a held first-bank Gate keeps its original release target across a mode change. A modal answer SHALL act once without a reveal hit or an underlying panel action. A second controller SHALL keep its own routes. A silent input SHALL not end the splash before accepted sound.

#### Scenario: Modal answer wins over panel entry
- **WHEN** a question or picker is visible and the SMC-PAD sends P8 while its panel is hidden
- **THEN** only the visible modal target receives one confirmation
- **AND** no panel cell runs, no hidden panel appears and a held Gate keeps its release

#### Scenario: Every reachable mode shows its badge
- **WHEN** keyboard and raw SMC-PAD runs enter each current input mode in separate runs
- **THEN** each run shows the mode badge and the same active target
- **AND** leaving the mode restores the previous input route
- **AND** Panel Back leaves marking and its marks before it hides the panel, as keyboard Esc does
