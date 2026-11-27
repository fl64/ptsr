## ADDED Requirements

### Requirement: A1 Setup shows the controls that apply

The overview SHALL show the Setup route when A1 is absent and Step Edit when A1 exists with no hits. The Setup footer SHALL show selection, value and exit controls, also with Help open. At 80x26, Setup SHALL use the work area below the title. In the overview and Setup, Full Help SHALL cover the command panel at both sizes. Full Help SHALL show Setup keys first and omit blocked edit actions. When Enter is bound, the current row SHALL determine its action.

#### Scenario: Absent A1 offers Setup
- **WHEN** the owner opens the A1 overview before A1 exists
- **THEN** the screen says that A1 is absent and names the Setup route
- **AND** if A1 exists with no hits, Step Edit remains available

#### Scenario: A1 Setup shows only applicable edit actions
- **WHEN** the owner opens Setup and reads the footer or F1
- **THEN** the instructions show the Setup controls and omit blocked pad editing actions
- **AND** the Setup controls appear at the start of Full Help and stay in the footer
- **AND** the command panel leaves Full Help readable when the panel is active

### Requirement: The A1 clear question shows its own controls

While the A1 clear question is open, its confirm and cancel controls SHALL take priority over the regular footer. The footer SHALL omit the underlying edit actions.

#### Scenario: A1 clear question hides underlying edit help
- **WHEN** the owner asks to clear A1
- **THEN** the question shows its answer controls and the footer omits the underlying edit actions

### Requirement: A1 mode help follows the active SMC panel

Setup and Step Edit SHALL show the SMC-PAD close gesture that applies. A verified hold SHALL use PAD BANK and P1. With the hold off or unavailable, the close cue SHALL name P1 and SHALL name the panel if it is hidden. At 80x26, active mode help SHALL use the full status line. When the A1 overview is idle, the line SHALL show the available E1 navigation role.

#### Scenario: A1 tells the owner how to close the current mode
- **WHEN** the owner uses an SMC-PAD in A1 Setup or Step Edit
- **THEN** the close cue follows the hold and panel state
- **AND** at 80x26 it leaves space for the pad action in Step Edit
