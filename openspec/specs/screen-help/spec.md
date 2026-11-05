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
Every screen SHALL reserve a fixed 3-row bottom help band without moving content above it when help changes.
Groups SHALL appear in this order: screen groups with the main step first, then BANK, REC and screens.
Groups that exceed the width SHALL disappear whole from the end: screens first, then REC.
F1 SHALL show all groups.

#### Scenario: Narrow terminal
- **WHEN** the terminal supports the kitty protocol, it is 140 columns wide and the chop screen is open
- **THEN** every group of the screen and BANK are visible, the P cap with "to pads" among the first items; the screens group drops off, F1 shows it

#### Scenario: One row on the pads screen
- **WHEN** the terminal supports the kitty protocol, it is 140 columns wide and the pads screen is open
- **THEN** the help has no key with a modifier; the name stays from the one-row help, the columns now take the rows their longest group needs

### Requirement: Help group columns

Each help group SHALL use a column of up to 3 `key text` rows, with its coloured name left of the first row.
Shorter groups SHALL put the name on the free first row above their items.
Columns SHALL be separated by `│`.
Groups over 3 items SHALL continue in a second column under the same name, without another group's columns between.

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

Held-modifier help SHALL show only working Ctrl/Alt chords or Shift commands and full-velocity pads, naming J/L roles where applicable.
Without Shift commands, Shift SHALL show only the pad hint.
Release SHALL restore usual help.
With modifier events, usual help SHALL show only plain keys.
Without them, Ctrl/Alt combinations and Chop Shift+H SHALL remain.
Text input SHALL keep typing priority without modifier help.

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

Each manual save request SHALL show `Session saved` after success or a current refusal reason after failure.
The notice SHALL last five seconds before the normal hint returns, unless another notice replaces it.
The default keyboard path SHALL remain Ctrl+S.
A learned controller button for `session.save` SHALL use the same save action and feedback.
A successful autosave SHALL stay silent.
A failed save SHALL also retain its persistent status after the notice ends.

#### Scenario: The manual save speaks
- **WHEN** the owner presses Ctrl+S and the save succeeds
- **THEN** the notice line shows `Session saved` and returns to the hint within five seconds

#### Scenario: The autosave is silent
- **WHEN** the session autosaves successfully after an idle period
- **THEN** no message shows and the hints and screen stay as they were

#### Scenario: Repeated manual refusals each explain the reason
- **WHEN** the owner requests manual saves twice while the write problem or read protection remains
- **THEN** each request shows a current refusal reason without clearing pending changes or showing `Session saved`

#### Scenario: Keyboard and controller save feedback agree
- **WHEN** keyboard and raw MIDI runs request the same save through Ctrl+S and a learned `session.save` button
- **THEN** both runs report the same success, write refusal or read-protection result
- **AND** both runs retain the same pending work and last saved file after failure

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

Help and recording hints SHALL use effective keys, including owner overrides.
Default Pads help SHALL name J/L values, U/O banks and Ctrl+C copy.
Default Chop help SHALL name H hits, Shift+H fewer hits, Ctrl+G normalize and Tab assign order while assigning.
F1 SHALL show controls omitted from bottom help.
Help SHALL NOT advertise unbound old chords as former actions.

#### Scenario: Pads help names values and new bank keys
- **WHEN** the owner opens F1 on Pads using default bindings
- **THEN** J/L change values, U/O change banks and Ctrl+C takes a copy

#### Scenario: Chop help names the moved commands
- **WHEN** the owner opens F1 on Chop using default bindings
- **THEN** it names H hits, Shift+H fewer hits and Ctrl+G normalize
- **AND** while assigning it names Tab for order, without advertising the ignored hit commands

### Requirement: Recording help follows effective keys

Default recording help SHALL name Alt+R microphone hold and Ctrl+T output.
Without key releases, it SHALL name microphone toggle instead of hold.
Owner recording keys SHALL override defaults in help.

#### Scenario: Microphone help follows release support
- **WHEN** the owner views microphone help or records with the default key
- **THEN** the key is Alt+R, with hold/release instructions when supported and toggle instructions otherwise

#### Scenario: Help respects an owner recording key
- **WHEN** the owner explicitly maps Ctrl+R to microphone recording
- **THEN** help names that effective key and does not claim it only shows a move notice

### Requirement: Copy help stays distinct from swap

A pending copy SHALL show `COPY · hit a pad` and cancellation instructions, distinct from swap, without moving the three-row help band.

#### Scenario: Copy in hand is visible without a mode redesign
- **WHEN** a copy is taken in hand on Pads in a small or large terminal
- **THEN** `COPY · hit a pad` and cancel instructions are visible, no swap is implied
- **AND** the grid, panel and strip do not move when the three-row help changes

### Requirement: Screen help keeps its current controls and learning groups

Pads, Settings, Files, Chop and FX SHALL keep their bottom/full help, action groups, effective keys, toggle states and modifier help.
The registry SHALL preserve group order, the three-row band and width rules.
Settings SHALL keep tab-specific help.

#### Scenario: Each registered screen keeps its effective help
- **WHEN** the owner opens each current screen with keys or equivalent MIDI actions and then opens full help
- **THEN** bottom help and full help keep that screen's existing action groups, effective keys and modifier rules
- **AND** owner overrides, the three-row band, group order and narrow-terminal behavior remain unchanged
- **AND** Settings help still follows its current tab and FX still names its block and parameter controls

### Requirement: Screen learning group compatibility

Screen learn SHALL keep the current Pads, Files, Chop and FX groups.
Pads edit SHALL retain PADS · EDIT.
Chop edit SHALL retain CHOP · EDIT with one bounds page.
FX SHALL keep its FX group without another edit group.
F10 SHALL NOT open screen learn on Settings.

#### Scenario: Screen learning keeps its existing entry points
- **WHEN** the owner opens screen learn through keyboard or raw MIDI actions on Pads, Files, Chop and FX in separate runs
- **THEN** each screen uses its existing screen group, Pads edit uses PADS · EDIT and Chop edit uses CHOP · EDIT
- **AND** FX has no separate edit group, Chop retains one bounds page and cancellation saves no pending binding
- **AND** the same learn action on Settings does not open screen learn or add a new learning path

### Requirement: MIDI list screen group compatibility

Settings SHALL keep its existing MIDI learning list, group names and order.
Opening Settings from Pads, Files, Chop or FX SHALL keep the corresponding screen group open.

#### Scenario: MIDI list keeps screen groups and their order
- **WHEN** the owner opens Settings from Pads, Files, Chop or FX through keyboard or raw MIDI actions in separate runs
- **THEN** the MIDI tab opens the corresponding screen group and keeps all existing group titles and ordering
- **AND** global, screen and edit assignments keep their existing learning scopes without renaming saved groups

### Requirement: Learn the save action from F12

The F12 MIDI tab SHALL list `session.save` as `save session` under GLOBAL.
The existing learning steps SHALL accept note and CC buttons, require Enter to save, and support Esc to cancel.
After restart, the confirmed assignment SHALL use the same save action and feedback as Ctrl+S on every input screen.
Learning SHALL NOT require a configuration edit or a new factory assignment.

#### Scenario: The save action appears in GLOBAL
- **WHEN** the owner opens the MIDI tab on F12 and opens GLOBAL
- **THEN** the action list contains one `save session` row for a button

#### Scenario: Save learning waits for confirmation
- **WHEN** the owner starts learning `save session` and presses an unused note or CC button
- **THEN** the row shows the pending assignment without changing the saved configuration
- **AND** Esc cancels the pending assignment without saving the session

#### Scenario: The learned save button survives restart
- **WHEN** the owner confirms a pending `save session` button and restarts the program
- **THEN** the saved GLOBAL assignment remains available without a configuration edit
- **AND** the button saves the current session on Pads, Files, Chop, FX and Settings with `Session saved` feedback

#### Scenario: The learned save button retries after write refusal
- **WHEN** a save fails through the button learned from F12 and the owner removes the write problem
- **THEN** pressing that button again saves the pending work and clears `Save failed`
- **AND** the next edit can autosave silently

### Requirement: Project transition refusal feedback

A refused project transition SHALL show the current save or target-read reason without a success notice or a new question.
The picker SHALL remain available for retry or cancellation.
Shared edits and picker entry SHALL use the existing keyboard and MIDI actions.
Picker selection, confirmation and text input SHALL keep their existing keyboard paths.

#### Scenario: A refused transition explains its reason
- **WHEN** an open or create request fails because source saving or target access fails
- **THEN** each request shows the current reason without `Session saved` or a project-open success notice
- **AND** no new mode or question replaces the picker

#### Scenario: Keyboard and MIDI project entry share transition safety
- **WHEN** keyboard and raw MIDI runs make the same edits and open the picker through their existing actions
- **AND** the owner selects or types the same target and confirms with the keyboard
- **THEN** both runs retain the same source work on refusal and save the same source work before success
