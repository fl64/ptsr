## ADDED Requirements

### Requirement: Help in plain words
The system SHALL make the help line say what the hands can do now, in plain words: at
most four groups named by the job (MOVE, CUT, BOUNDS, PADS, EDIT…), the most used first,
and after them the screens (F-keys) and the help (F1);
every item is a key cap and a verb ("J L cursor", "M cut"); the direction letters are
shown, the arrows are named once in F1; in an edit mode the line shows only the edit keys
and the way out.

#### Scenario: The chop line reads as jobs
- **WHEN** the chop screen is open in the listen mode
- **THEN** the help line has the groups MOVE (J L cursor, I K zoom), CUT (M cut, Y equal
  parts), BOUNDS (Ctrl J L left, Alt J L right) and PADS (P to pads, Enter edit), and no
  item is longer than a key and two words

#### Scenario: The edit line
- **WHEN** the pad edit mode is open on the ENV page
- **THEN** the help line shows only EDIT: I K the page and the parameter, J L the value,
  a pad hit to listen, Enter or Esc to close

## MODIFIED Requirements

### Requirement: Help for a held modifier
The system SHALL, while a modifier is held, show in the help line only the keys that work with that modifier on the open screen and nothing else: Ctrl shows the Ctrl combos, Alt shows the Alt combos (the right bound of a slice), Shift shows that pads play at full velocity; each says what the modifier does with `J` `L`. The usual line comes back on release. Where the terminal reports modifiers (the kitty protocol), the usual line SHALL show only keys without modifiers; elsewhere, where a held modifier cannot be seen, it SHALL keep the Ctrl and Alt combos.

#### Scenario: Hold Ctrl
- **WHEN** the pads screen is open and the owner holds Ctrl
- **THEN** the help line shows only the Ctrl combos of the pads screen, among them R "mic: hold", T "output", 1…8 "bank", and no key without Ctrl; on release the usual line is back

#### Scenario: Hold Alt
- **WHEN** the chop screen is open and the owner holds Alt
- **THEN** the help line shows only the Alt combos of the chop screen, among them J L "right bound", and no key without Alt; on release the usual line is back

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

### Requirement: Help line fits the terminal
The system SHALL keep a band of 3 rows at the bottom of every screen for the help line,
whatever the help says now: the screen above the band never moves when the help changes
(an edit mode opens, the assign starts, a modifier is held). The help fills the band from
its first row and continues on the next rows when it does not fit on one; the key of the
main step of the screen comes first.

#### Scenario: Narrow terminal
- **WHEN** the terminal is 140 columns wide and the chop screen is open
- **THEN** every group of the help line is visible, the P cap with "to pads" among the first items

#### Scenario: One row on the pads screen
- **WHEN** the terminal supports the kitty protocol, it is 140 columns wide and the pads screen is open
- **THEN** the help line fills the first row of the band and has no key with a modifier

#### Scenario: Nothing jumps
- **WHEN** the owner opens the edit mode of a pad, holds Ctrl, lets it go and presses `Esc`
- **THEN** the grid, the panel and the strip stay on the same rows all the time; only the
  help band changes

