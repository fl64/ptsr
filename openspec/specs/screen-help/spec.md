# screen-help Specification

## Purpose
Help at the bottom of every screen with current toggle states and a next-step hint; full help on F1.

## Requirements

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

### Requirement: First steps hint
The system SHALL, while no pad holds a sample, say on the pads screen how to put one there; short-lived messages (a controller connects, audio is unavailable) do not remove the hint — it is back as soon as the message is gone.

#### Scenario: Empty session
- **WHEN** the program starts with no samples on pads and there is no other message
- **THEN** the hint line says "Pads are empty: F6 opens files, Enter takes one, a pad hit puts it there."

#### Scenario: A message passes
- **WHEN** no pad holds a sample, the program shows "MIDI connected" and the message fades out
- **THEN** the first-steps hint is on the pads screen again

#### Scenario: Done with the hint
- **WHEN** the first pad gets a sample
- **THEN** the hint is not shown anymore

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

### Requirement: Help line per settings tab
The system SHALL list on the settings screen only the keys that work on the open tab.

#### Scenario: Audio tab
- **WHEN** the Audio tab is open
- **THEN** the help line has no "learn" and no "clear"

### Requirement: Screens group in key order
The system SHALL list the screen keys of the help line in ascending key order: help, chop, files, settings.

#### Scenario: The last group of the help line
- **WHEN** any screen shows its help line
- **THEN** the screens group reads F1, F3, F6, F12 in that order

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

### Requirement: Manual save feedback
The system SHALL show "Session saved" for 5 seconds after a manual save (`Ctrl+S`); the
autosave stays silent, and the hint line comes back when the flash is gone.

#### Scenario: The manual save speaks
- **WHEN** the owner presses `Ctrl+S`
- **THEN** the notice line shows "Session saved" and is back to the hint within 5 seconds

#### Scenario: The autosave is silent
- **WHEN** the session autosaves after an idle period
- **THEN** no message shows; the hints and the screen stay as they were

### Requirement: The pads switch in the help
The system SHALL show in the help line of the pads, files and chop screens the pads switch with its state: `O pads: shown` or `O pads: hidden` among the Ctrl keys, which show while Ctrl is held; without the kitty protocol, where a held Ctrl is not seen, the full help (F1) lists the switch: the usual line of the chop screen has no room left for it within the three help rows.

#### Scenario: The help names the state
- **WHEN** the pads are hidden and the owner holds Ctrl on F6
- **THEN** the help line says `pads: hidden` next to `O`

### Requirement: The chop hint follows the work
The system SHALL say in the hint line of the chop screen the next step for the state of the chop: no cuts yet — `Y` or `Space` and `M`; cuts and no slice on a pad — `P` lays them on pads; slices on pads — the bounds move with `Ctrl`/`Alt` and `J` `L` and the pads follow; no hint in the edit.

#### Scenario: The hint after a layout
- **WHEN** the slices of a chop lie on pads
- **THEN** the hint says how to move a bound, not how to start a chop

### Requirement: Combos read as combos without kitty

Without the extended keyboard protocol, the system SHALL write modifier keys as full
combinations, consistently: Ctrl+Z, Alt+R, Shift+H, Ctrl+J/L and Alt+J/L. With modifier
events, held-modifier help SHALL keep the modifier in its group and show the key caps.

#### Scenario: One way to write Ctrl
- **WHEN** Chop is open without the extended keyboard protocol
- **THEN** undo reads Ctrl+Z like Ctrl+T, bounds read Ctrl+J/L and Alt+J/L, microphone
  reads Alt+R, and the reverse hit command reads Shift+H

### Requirement: The cut removal reads as such
The system SHALL name `Backspace` on the chop screen "remove cut" in the help line and in
F1, not "join".

#### Scenario: The owner looks for a way to remove a cut
- **WHEN** the chop screen is open in the listen mode
- **THEN** the help line has `Bksp remove cut` in the CUT group

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
