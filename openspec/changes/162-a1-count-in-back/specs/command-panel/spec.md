## MODIFIED Requirements

### Requirement: SMC-PAD second bank opens a command panel

Without the PAD BANK hold shortcut, the Master profile SHALL map 16 second-bank notes to panel cells in physical rows. An owner binding MAY replace a cell outside a question. A first panel-bound press while hidden SHALL only reveal the panel, except for command P1 during count-in on the A1 overview. That press SHALL cancel count-in and keep the panel hidden. Later presses SHALL run one visible cell. Releases SHALL run no cell. A first-bank press SHALL hide the panel and keep its pad action and release. With the shortcut on, the hold SHALL decide.

#### Scenario: First second-bank hit only reveals the panel
- **WHEN** the owner starts transport and sends a second-bank press and release with the shipped panel binding while the panel is hidden
- **THEN** the panel appears in the pad-grid area and no cell action runs
- **AND** transport and sounding voices continue without a new hit or stop

#### Scenario: Two second-bank presses keep input order
- **WHEN** two second-bank presses reach the MIDI dispatcher before one UI redraw
- **THEN** the first reveals the panel without action and the second runs its visible cell once
- **AND** either release runs no cell

#### Scenario: First-bank hit leaves the panel and keeps its Gate
- **WHEN** the panel is visible and a first-bank Gate pad is pressed, then released after a bank or screen change
- **THEN** the press hides the panel and sounds once at its normal physical position
- **AND** the original release ends that Gate once without releasing another pad

#### Scenario: P1 cancels count-in before panel reveal
- **WHEN** command P1 reaches the A1 overview during count-in with a hidden panel and the PAD BANK hold is off or unavailable
- **THEN** the first press cancels count-in, the overview stays open and the panel stays hidden
- **AND** a P1 press in the playing layer still plays its pad
