## MODIFIED Requirements

### Requirement: Help for a held modifier

The system SHALL, while a modifier is held, show only keys that work with that modifier
on the open screen: Ctrl chords, Alt chords or Shift commands plus full-velocity pads.
Each SHALL say what it does with J/L where applicable. Without a Shift command on that
screen, Shift SHALL show only the full-velocity pad hint. The usual line SHALL return
on release. With extended modifier events, the usual line SHALL show unmodified keys
only; without them, it SHALL retain Ctrl/Alt combinations and show Shift+H on Chop.
A text input SHALL keep typing priority and SHALL NOT switch to modifier help.

#### Scenario: Hold Ctrl
- **WHEN** Pads is open with default bindings and the owner holds Ctrl
- **THEN** help shows only Ctrl chords, including T for output, C for copy and 1…8 for banks
- **AND** it shows no R microphone action or unmodified keys; release restores usual help

#### Scenario: Hold Alt
- **WHEN** Chop is open and the owner holds Alt
- **THEN** help shows only Alt chords, including J/L for the right bound and R for microphone
- **AND** release restores usual help

#### Scenario: Hold Shift
- **WHEN** Pads is open and the owner holds Shift
- **THEN** help shows only full-velocity pads and a pad hit has full velocity as before

#### Scenario: A lost release
- **WHEN** Ctrl is held, focus is lost and the next event is a pad key without Ctrl
- **THEN** usual help returns

#### Scenario: Typing a filter
- **WHEN** the filter input is open and the owner holds Shift to type a capital
- **THEN** the help line does not change

#### Scenario: No kitty protocol
- **WHEN** the terminal does not report modifiers
- **THEN** the usual help shows the working Ctrl/Alt combinations without waiting for a held modifier

#### Scenario: Shift help names reverse hit chopping
- **WHEN** Chop is in listen mode and the owner holds Shift
- **THEN** help names H for fewer hit cuts and full-velocity pads
- **AND** it does not list the unshifted H command as the action of Shift+H

### Requirement: Combos read as combos without kitty

Without the extended keyboard protocol, the system SHALL write modifier keys as full
combinations, consistently: Ctrl+Z, Alt+R, Shift+H, Ctrl+J/L and Alt+J/L. With modifier
events, held-modifier help SHALL keep the modifier in its group and show the key caps.

#### Scenario: One way to write Ctrl
- **WHEN** Chop is open without the extended keyboard protocol
- **THEN** undo reads Ctrl+Z like Ctrl+T, bounds read Ctrl+J/L and Alt+J/L, microphone
  reads Alt+R, and the reverse hit command reads Shift+H

## ADDED Requirements

### Requirement: Help follows the cleaned keymap

The system SHALL derive help and recording hints from effective keys, including owner
overrides. With default bindings, Pads help SHALL show J/L values, U/O banks and Ctrl+C
copy; Chop help SHALL show H hits, Shift+H fewer hits, Ctrl+G normalize and Tab assign
order while assigning. Recording help SHALL use Alt+R for microphone hold and Ctrl+T
for output; without releases it SHALL say microphone toggle instead of hold. F1 SHALL
show all controls when bottom groups do not fit. Help SHALL NOT advertise an unbound
old chord as its former action. A pending copy SHALL show `COPY · hit a pad` and how
to cancel, distinct from a swap, with no jump of the three-row help band.

#### Scenario: Pads help names values and new bank keys
- **WHEN** the owner opens F1 on Pads using default bindings
- **THEN** J/L change values, U/O change banks and Ctrl+C takes a copy

#### Scenario: Chop help names the moved commands
- **WHEN** the owner opens F1 on Chop using default bindings
- **THEN** it names H hits, Shift+H fewer hits and Ctrl+G normalize
- **AND** while assigning it names Tab for order, without advertising the ignored hit commands

#### Scenario: Microphone help follows release support
- **WHEN** the owner views microphone help or records with the default key
- **THEN** the key is Alt+R, with hold/release instructions when supported and toggle instructions otherwise

#### Scenario: Help respects an owner recording key
- **WHEN** the owner explicitly maps Ctrl+R to microphone recording
- **THEN** help names that effective key and does not claim it only shows a move notice

#### Scenario: Copy in hand is visible without a mode redesign
- **WHEN** a copy is taken in hand on Pads in a small or large terminal
- **THEN** `COPY · hit a pad` and cancel instructions are visible, no swap is implied
- **AND** the grid, panel and strip do not move when the three-row help changes
