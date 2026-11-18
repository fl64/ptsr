## MODIFIED Requirements

### Requirement: SMC-PAD second bank opens a command panel

Without the PAD BANK hold shortcut, the Master profile SHALL map 16 second-bank notes to panel cells in the first bank's physical rows. An owner binding MAY replace a cell outside a question. The first panel-bound press while hidden SHALL reveal the panel without an action. Later presses SHALL run one visible cell. Releases SHALL run no cell. A first-bank press SHALL hide the panel and keep its pad action and original release. With the shortcut on, the PAD BANK hold SHALL decide the panel instead.

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

### Requirement: Visible questions use the two agreed pads

While a question is visible, physical P7 SHALL cancel and P8 SHALL confirm on press, even with owner bindings and a hidden panel. With the hold shortcut on, they SHALL answer from either note bank without a hold, and the other pads SHALL stay silent. Without it, other second-bank cells SHALL not reveal or act. PLAY and STOP SHALL never answer the question. Their context transport roles MAY act. Keyboard and learned answers SHALL keep their rules.

#### Scenario: P7 and P8 answer a visible question on the first hit
- **WHEN** a question is visible and the owner presses P7 or P8 from a hidden panel in separate runs
- **THEN** P7 cancels and P8 confirms through the normal question action
- **AND** neither press first reveals the panel or acts as an ordinary cell

#### Scenario: A question takes its two pads without a hold
- **WHEN** a question is visible, the hold shortcut is on and PAD BANK is not held
- **THEN** P7 and P8 from either note bank answer the question
- **AND** a hit on another pad stays silent and the question stays

#### Scenario: A panel action followed by P8 confirms only the question
- **WHEN** a destructive panel cell opens a question and P8 arrives before the next redraw
- **THEN** P8 confirms only that visible question once
- **AND** no earlier cell action repeats and no second target changes

#### Scenario: PLAY and STOP do not answer a question
- **WHEN** a question is visible and PLAY or STOP sends a tap
- **THEN** the context playback action runs where available
- **AND** the question and its target stay unchanged
