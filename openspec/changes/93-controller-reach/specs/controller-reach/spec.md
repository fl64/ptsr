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

### Requirement: Input exceptions stay explicit

The only controller-only steps SHALL be PAD BANK selection, its status mark and page navigation within its panel. The only keyboard-only operations SHALL be `pad.rename`, `browser.filter`, a typed new-project name, `app.quit` and keymap edits. `app.quit` SHALL remain unavailable for MIDI learn. No other exception SHALL enter the inventory without an owner decision.

#### Scenario: Text entry and quit keep their exceptions
- **WHEN** the reach inventory examines pad naming, file filtering, a typed new-project name, keymap edits and quit
- **THEN** it marks only those operations as keyboard-only
- **AND** MIDI learn does not offer `app.quit`

### Requirement: Controller routes keep short press paths

A current frequent action SHALL have a direct SMC-PAD button, pad or knob route. Each other non-text action SHALL need at most two presses from the visible main panel, apart from knob turns and target pad hits. The reach table SHALL state the count from a hidden panel separately. That count SHALL include PAD BANK selection, a reveal hit and the action hit. It SHALL name a route and both counts for every action story in `docs/USER-STORIES.md`.

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
