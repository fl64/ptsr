## ADDED Requirements

### Requirement: The FX frame shows return and mute controls

The FX frame SHALL show the level and mute state of the current return. The keyboard SHALL select Return in the existing parameter list and use the normal value controls. M SHALL toggle its mute. The SMC-PAD SHALL use E5 for the current return. The panel SHALL show the mute action on P13. The state and hints SHALL fit both supported terminal sizes.

#### Scenario: Both inputs show the same FX return feedback

- **WHEN** the owner changes a return or mutes it with the keyboard or SMC-PAD
- **THEN** the selected slot shows its changed level or mute state
- **AND** the owner can see how to select the other slot and return to the pad screen
