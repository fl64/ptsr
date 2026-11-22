## ADDED Requirements

### Requirement: Old projects keep their pad response

The project SHALL save a default pad response and per-pad Pan, Fine tune, Curve, Hold and Velocity overrides. An old project SHALL keep its linear response and its prior sound, including empty pad hits. An assigned pad SHALL keep its controls when the owner replaces its sample. A first assignment to a new pad SHALL use the new response for fields that have no explicit pad setting.

#### Scenario: Old pad playback survives a save and reopen

- **WHEN** the owner opens and saves an older project with assigned and empty pad hits in A1
- **THEN** both kinds of hit keep the old linear response after reopen
- **AND** a newly assigned pad uses the new defaults without changing the others
