## ADDED Requirements

### Requirement: Settings help follows its active tab

The live hint and F1 SHALL describe the active Settings tab. Input SHALL describe viewing MIDI input. Paths SHALL name row selection and Enter to open the folder picker. The folder picker SHALL name browse, select and cancel controls. The Audio and MIDI tabs SHALL describe their own controls.

#### Scenario: Settings help follows the current controls

- **WHEN** the owner visits Audio, Input, MIDI and Paths and opens the folder picker
- **THEN** the live hint and F1 name only controls that apply to the current tab or picker
