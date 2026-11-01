## ADDED Requirements

### Requirement: The change scope in the help line
The system SHALL show the change scope in the help line of the pads screen with the mark
toggle, as `mark: off · scope: pad Q`, `scope: 3 marked`, `scope: bank 2` or
`scope: all pads`, so the owner sees what the next change hits without opening the pad
edit. It stays in the help line of a terminal that hides the `Ctrl` combos until `Ctrl`
is held.

#### Scenario: Scope in the help line
- **WHEN** the pads screen is open on bank 2 and the owner presses `Ctrl+A` until the
  scope is the bank
- **THEN** the help line says `mark: off · scope: bank 2` at the mark key, also in a
  terminal with the kitty protocol; with the scope back on the pad, after marking three
  pads it says `mark: on, hit pads · scope: 3 marked`

### Requirement: Plain words for no key releases
The system SHALL, in a terminal that gives no key releases, mark the status for the
session with `no key releases: gate pads play one-shot` and add to the full help (F1) a
row that says Ghostty, kitty, WezTerm and foot give key releases. With key releases there
is neither the marker nor the row.

#### Scenario: Plain words for no key releases
- **WHEN** the terminal gives no key releases
- **THEN** the status says `no key releases: gate pads play one-shot`

#### Scenario: F1 says where releases come from
- **WHEN** the terminal gives no key releases and the owner opens F1
- **THEN** F1 has a row that names Ghostty, kitty, WezTerm and foot; in a terminal with
  key releases the row is absent
