## MODIFIED Requirements

### Requirement: On-screen help
The system SHALL show at the bottom of every screen help grouped by actions, with the current state of a toggle only when the screen does not show that state elsewhere, and a hint for the next step; keys are drawn as key caps, distinct from their text; full help opens on F1.

#### Scenario: Toggle in the help
- **WHEN** the files screen is open and the preview is switched off
- **THEN** the help shows "preview: off", not only the key

#### Scenario: State shown once
- **WHEN** the pads screen is open and the selected pad is in gate mode at −3 dB
- **THEN** the pad panel shows "mode gate" and "volume -3 dB", and the help line says only "mode" and "volume" next to their keys

#### Scenario: Keys look like keys
- **WHEN** any screen is open
- **THEN** every key of the help line is drawn as a cap with its own background, the text next to it is plain

### Requirement: Help line fits the terminal
The system SHALL show the whole help line: when it does not fit on one row it continues on a second row, and the key of the main step of the screen comes first.

#### Scenario: Narrow terminal
- **WHEN** the terminal is 140 columns wide and the chop screen is open
- **THEN** every group of the help line is visible, the Enter cap with "to pads" among the first items

#### Scenario: One row on the pads screen
- **WHEN** the terminal supports the kitty protocol, it is 140 columns wide and the pads screen is open
- **THEN** the help line takes one row and has no key with a modifier

### Requirement: Grouped full help
The system SHALL show the full help (F1) grouped under headings by purpose, with keys of one kind on one row, fitting the screen; no line is cut and a text does not repeat its own key.

#### Scenario: Banks in one row
- **WHEN** the owner opens F1 on the pads screen
- **THEN** banks 1–8 are one row "Ctrl+1…8", rows are under headings such as PADS, BANK, SCREENS

#### Scenario: Pairs in one row
- **WHEN** the owner opens F1 on the settings screen
- **THEN** "← →" is one row, "↑ ↓" is one row, "PgUp PgDn" is one row

#### Scenario: Nothing is cut
- **WHEN** the owner opens F1 on any screen of a 140×34 terminal
- **THEN** every row is whole, and the microphone row reads "record the microphone: hold" next to its key

## ADDED Requirements

### Requirement: Help for a held modifier
The system SHALL, while a modifier is held, show in the help line only the keys that work with that modifier on the open screen and nothing else: Ctrl shows the Ctrl combos, Shift shows that pads play at full velocity. The usual line comes back on release. Where the terminal reports modifiers (the kitty protocol), the usual line SHALL show only keys without modifiers; elsewhere, where a held Ctrl cannot be seen, it SHALL keep the Ctrl combos.

#### Scenario: Hold Ctrl
- **WHEN** the pads screen is open and the owner holds Ctrl
- **THEN** the help line shows only the Ctrl combos of the pads screen, among them R "mic: hold", T "output", 1…8 "bank", and no key without Ctrl; on release the usual line is back

#### Scenario: Hold Shift
- **WHEN** the owner holds Shift
- **THEN** the help line shows only that pads play at full velocity, and a pad hit with Shift plays at full velocity as before

#### Scenario: A lost release
- **WHEN** the owner holds Ctrl, the terminal loses focus and the next event is a pad key without Ctrl
- **THEN** the usual help line is back

#### Scenario: Typing a filter
- **WHEN** the filter line is open and the owner holds Shift to type a capital letter
- **THEN** the help line does not change

#### Scenario: No kitty protocol
- **WHEN** the terminal does not report modifiers
- **THEN** the usual line shows the Ctrl combos as before
