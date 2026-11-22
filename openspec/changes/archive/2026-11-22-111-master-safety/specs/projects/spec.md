## ADDED Requirements

### Requirement: Projects keep the shared effect returns

The project SHALL keep the level and mute state per shared FX slot. An existing project with no saved return controls SHALL use 100 percent and unmuted for both slots.

#### Scenario: Shared FX return controls survive a project reload

- **WHEN** the owner saves two different shared FX returns with one muted and opens the project again
- **THEN** each slot restores its level and mute state
- **AND** an older project without these values opens at unity with both slots unmuted
