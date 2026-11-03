## ADDED Requirements

### Requirement: Pads playback badge

On the Pads screen (F2), the system SHALL show a PLAY badge next to the screen title.
It SHALL occupy 10 terminal cells including padding: ` PLAY     ` at rest and
` PLAY ▶   ` while at least one real pad voice of the current ready project plays.
Idle text SHALL be dim on the normal background; playback SHALL use the existing
blue background and the play icon. Following title content SHALL keep its columns.
The badge SHALL include all banks, not only the selected pad or visible bank, and
SHALL work with pad edit open or closed, with the grid hidden or shown, and alongside
the existing MARK plate without changing that plate's meaning or appearance.

The badge SHALL follow actual audio playback, including a gate's release fade, not
held keys, hit flashes or queued requests. Previews, an effect tail alone, rejected
or blocked hits and old-project voices SHALL NOT light it. A project that is not ready
or missing audio SHALL show idle. Keyboard pads, raw MIDI notes and learned pad buttons
SHALL give the same result through their production routes. This visual addition
SHALL NOT change sound, input routing, keys, transport, F3 or saved data.

#### Scenario: Pads PLAY badge follows keyboard and MIDI playback
- **WHEN** a keyboard pad, raw MIDI note or learned pad button triggers a pad on F2
  and the real audio callback advances it
- **THEN** PLAY lights blue with the play icon until the voice ends naturally or its
  gate release fade finishes, then returns to dim without the icon
- **AND** a queued hit alone or a held key after natural sound end does not keep it lit

#### Scenario: Pads PLAY badge follows all sounding pads
- **WHEN** several pad voices play, the selected pad or bank changes, or the owner
  returns to F2, opens pad edit or hides the grid while a pad still plays
- **THEN** PLAY stays lit until the last pad voice ends, regardless of selection or bank

#### Scenario: Pads PLAY badge ignores silent requests and old projects
- **WHEN** question or learn blocks a hit, its audio queue rejects it, the project is
  not ready, audio is absent, or the project changes while an old pad still sounds
- **THEN** those requests and old voices do not light PLAY
- **AND** a valid current-project voice still lights it until that voice ends

#### Scenario: Pads PLAY badge ignores previews and effect tails
- **WHEN** no pad voice remains but a sample preview or an effect tail still sounds
- **THEN** PLAY is dim without the icon even though the output is not silent

#### Scenario: Pads PLAY badge has fixed width
- **WHEN** F2 changes between silence and pad playback with pad edit open or closed,
  the grid hidden or shown, and a MARK plate present or absent
- **THEN** PLAY keeps its 10-cell width and the following title content does not move
- **AND** MARK keeps its existing text and colors and the shared screen frame stays put
