## ADDED Requirements

### Requirement: The cut removal reads as such
The system SHALL name `Backspace` on the chop screen "remove cut" in the help line and in
F1, not "join".

#### Scenario: The owner looks for a way to remove a cut
- **WHEN** the chop screen is open in the listen mode
- **THEN** the help line has `Bksp remove cut` in the CUT group

## MODIFIED Requirements

### Requirement: Help line fits the terminal
The system SHALL keep a band of 3 rows at the bottom of every screen for the help line,
whatever the help says now: the screen above the band never moves when the help changes
(an edit mode opens, the assign starts, a modifier is held). The help stands in columns:
every group is a column of up to 3 rows of `key text`, the group name in colour left of
its first row (a group shorter than the band puts its name on the free first row above
its items), the columns split by `│`; a group of more than 3 items takes a second
column under the same name, and a group never breaks between the columns of another
group. The order is the same on every screen: the screen's own groups first, the main
step of the screen first among them, then BANK, REC and the screens. When the columns do
not fit the width, the last groups drop off whole, the screens first, then REC; F1 shows
them all.

#### Scenario: Narrow terminal
- **WHEN** the terminal supports the kitty protocol, it is 140 columns wide and the chop screen is open
- **THEN** every group of the screen and BANK are visible, the P cap with "to pads" among the first items; the screens group drops off, F1 shows it

#### Scenario: One row on the pads screen
- **WHEN** the terminal supports the kitty protocol, it is 140 columns wide and the pads screen is open
- **THEN** the help has no key with a modifier; the name stays from the one-row help, the columns now take the rows their longest group needs

#### Scenario: Columns on the pads screen
- **WHEN** the terminal is 140 columns wide and the pads screen is open
- **THEN** the help shows the columns PAD, CHANGE, BANK, REC and the screens in this order, every item of a group under the one before

#### Scenario: The same order on every screen
- **WHEN** the owner opens the pads, the chop and the files screens in turn
- **THEN** BANK stands right after the screen's own groups on each of them

#### Scenario: Too narrow for all
- **WHEN** the terminal is 100 columns wide and the files screen is open
- **THEN** the screen's groups and BANK are whole, the screens group is gone, and nothing is cut in the middle of a group

#### Scenario: Nothing jumps
- **WHEN** the owner opens the edit mode of a pad, holds Ctrl, lets it go and presses `Esc`
- **THEN** the grid, the panel and the strip stay on the same rows all the time; only the
  help band changes
