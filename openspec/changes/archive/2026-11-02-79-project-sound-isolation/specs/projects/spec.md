## ADDED Requirements

### Requirement: Project sounds stay separate

The system SHALL play only the sounds of the open project on new pad hits. An empty or
missing pad SHALL be silent. Loading from an earlier project SHALL NOT change the open
project, even if it finishes after the switch. Each pad SHALL use the settings of the
open project. These rules apply to keyboard and MIDI hits. On a project switch, sounds
already playing from the old project SHALL fade out without a click. Until the new project
is ready, pad hits SHALL be silent and SHALL NOT play later.

#### Scenario: Empty pads after a project switch
- **WHEN** the owner switches from a project with samples and slices to a new empty project
  and hits the same pads
- **THEN** the pads are silent, even when the audio command queue was full during the switch

#### Scenario: A late sample from the old project
- **WHEN** a sample load from the old project finishes after a switch to an empty project
- **THEN** the empty pad stays empty and silent

#### Scenario: Playing sounds stop at a project switch
- **WHEN** a pad sounds and the owner opens a different project
- **THEN** the old sound fades out without a click and stays silent

#### Scenario: Hits while the project opens
- **WHEN** the owner hits pads while the new project is not ready
- **THEN** the hits are silent and do not play later

#### Scenario: Return to a project with sounds
- **WHEN** the owner switches from a project with samples and slices to an empty project
  and then returns to the first project
- **THEN** its pads play their own samples and slices with their saved settings
