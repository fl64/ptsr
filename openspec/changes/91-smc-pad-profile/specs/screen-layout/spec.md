# screen-layout Delta

## ADDED Requirements

### Requirement: SMC-PAD controller view follows the physical surface

The F1 controller view SHALL show the first hardware pad bank in the same top-to-bottom order as the screen pads. It SHALL show the second hardware pad bank as distinct controls without command labels until the panel change. It SHALL show both sets of eight encoders and the five buttons in their physical groups. The view SHALL name each available current role and dim a control without a role. It SHALL not show SHIFT, PAD BANK or KNOB BANK as host commands.

#### Scenario: Controller view matches both pad banks
- **WHEN** the owner opens the SMC-PAD controller view and switches the local PAD BANK
- **THEN** the view identifies notes 48 through 51 as the top row and notes 36 through 39 as the bottom row of the first bank
- **AND** the second bank has the same physical row order and no active command label

#### Scenario: Encoder roles follow the current screen
- **WHEN** the owner views Pads, Files and Chop with the SMC-PAD selected
- **THEN** both encoder sets name the available roles for that screen
- **AND** unavailable roles are dim, while the controller status shows when encoder setup is incomplete
