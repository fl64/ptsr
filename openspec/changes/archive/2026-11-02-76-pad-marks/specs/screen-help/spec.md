## MODIFIED Requirements

### Requirement: The change scope in the help line
The system SHALL show the change scope in the help line of the pads screen with the mark
toggle, as `mark: off · scope: pad Q` or `scope: 17 marked`, so the owner sees what the
next change hits without opening the pad edit, and SHALL name `Ctrl+A` as `mark bank` in
the full help (F1). It stays in the help line of a
terminal that hides the `Ctrl` combos until `Ctrl` is held.

#### Scenario: Scope in the help line
- **WHEN** the pads screen is open on bank 2 and the owner presses `Ctrl+A`
- **THEN** the help line says `mark: on, hit pads · scope: 16 marked` at the mark key, also
  in a terminal with the kitty protocol; after `Esc` it says `mark: off · scope: pad Q`

#### Scenario: The mark keys in the full help
- **WHEN** the owner opens the full help (F1) on the pads screen
- **THEN** it names `Ctrl+A mark bank`, and no key named "scope"
