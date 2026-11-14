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

Each manual save request SHALL show `Session saved` for five seconds after success, or an Error notice with the current refusal reason after failure.
Ctrl+S and a learned `session.save` button SHALL use the same action and feedback.
Successful autosave SHALL stay silent.
Notice lifetime or acknowledgement SHALL NOT clear pending work, read protection or persistent save failure.

#### Scenario: The manual save speaks
- **WHEN** the owner presses Ctrl+S and the save succeeds
- **THEN** the notice line shows `Session saved` for five seconds before returning to the hint unless another notice takes priority

#### Scenario: The autosave is silent
- **WHEN** the session autosaves successfully after an idle period
- **THEN** no message shows and the hints and screen stay as they were

#### Scenario: Repeated manual refusals each explain the reason
- **WHEN** the owner requests manual saves twice while the write problem or read protection remains
- **THEN** each request shows its current refusal as an Error and adds its own history entry
- **AND** pending changes remain and no `Session saved` result appears

#### Scenario: Keyboard and controller save feedback agree
- **WHEN** keyboard and raw MIDI request the same save through Ctrl+S and a learned `session.save` button
- **THEN** both report the same success, write refusal or read-protection result and notice level
- **AND** both retain the same pending work and saved bytes after failure

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

The system SHALL show the next edit scope in the Pads help line with the mark toggle. It SHALL say `edit: this pad`, `edit: marked (N)`, or `edit: bank` for the applicable scope. It SHALL name `Ctrl+A` as `mark bank` in F1. The scope SHALL remain visible when a terminal hides Ctrl combinations until the owner holds Ctrl.

#### Scenario: Scope in the help line
- **WHEN** Pads selects one pad, marks three pads, then marks the bank and clears marks
- **THEN** the help line shows `edit: this pad`, `edit: marked (3)`, `edit: bank`, and `edit: this pad` in that order
- **AND** the mark state stays visible and the same pad operations affect the same pads

#### Scenario: The mark keys in the full help
- **WHEN** the owner opens F1 on Pads
- **THEN** it names `Ctrl+A mark bank`, and no key named `scope`

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

Default recording help SHALL name Ctrl+R pattern recording, Alt+R microphone hold and Ctrl+T output. Without releases, microphone help SHALL name toggle. Owner recording keys SHALL override defaults in help. Pattern recording SHALL remain distinct from audio capture. No help SHALL advertise an unimplemented SMC-PAD hold or factory role.

#### Scenario: Microphone help follows release support
- **WHEN** the owner views microphone help or records with the default key
- **THEN** the key is Alt+R, with hold/release instructions when supported and toggle instructions otherwise

#### Scenario: Help respects an owner recording key
- **WHEN** the owner explicitly maps Ctrl+R to microphone recording
- **THEN** help names that effective key and does not claim it only shows a move notice or records A1

#### Scenario: Full help distinguishes the three recording targets
- **WHEN** F1 opens with default keys on each current screen
- **THEN** Ctrl+R names A1 recording, Alt+R names microphone and Ctrl+T names output
- **AND** owner overrides and `none` change those labels without an obsolete move notice

### Requirement: Copy help stays distinct from swap

A pending copy SHALL show `COPY · hit a pad` and cancellation instructions, distinct from swap, without moving the three-row help band.
The shared mode badge SHALL identify copy even when compact help omits its detail.
Existing copy placement, confirmation and cancellation actions SHALL remain unchanged.

#### Scenario: Copy in hand is visible without a mode redesign
- **WHEN** a copy is taken in hand on Pads in a small or large terminal
- **THEN** `COPY · hit a pad` or its compact COPY badge and cancel instructions are visible without implying swap
- **AND** grid, panel and strip rows stay fixed when the help changes

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

### Requirement: Notice levels and origins

Transient notices SHALL use Done, Info, Warn or Error and record Keyboard, the actual Device, or System as origin.
They SHALL use `✓` cyan, `·` muted, `!` magenta and `✗` bold magenta respectively.
Existing transient and startup messages SHALL enter the same model without inventing device identity.
Text-input prompts, ongoing recording/loading state and persistent save status SHALL retain their separate precedence and meaning.

#### Scenario: Notice appearance identifies its level
- **WHEN** a notice of each level appears in a separate fixture
- **THEN** its glyph, text and style identify the level without relying on color alone
- **AND** the stored origin identifies the keyboard, actual device or system

#### Scenario: Each produced result keeps its meaning
- **WHEN** an existing operation succeeds, refuses the request or completes after a delay
- **THEN** its notice level comes from that result, not from matching words in its text
- **AND** loading completion removes only its own ongoing loading state, not an unrelated Error

#### Scenario: Device notices retain the actual event source
- **WHEN** two controllers alternate commands, deferred pad actions and parameter changes through their production profiles
- **THEN** each resulting notice retains that event's actual device origin, without borrowing the other controller's identity
- **AND** a keyboard result records Keyboard while startup, loader and connection events record System
- **AND** source metadata does not change velocity, command order, epochs, releases or audio routing

#### Scenario: Input prompts retain typing priority
- **WHEN** a filter, name or picker text line is open while background notices arrive
- **THEN** its prompt and cursor stay usable and pad keys still type without playing
- **AND** the notices enter history without replacing the text input

#### Scenario: Startup warnings enter history once
- **WHEN** startup produces warnings or errors and frames redraw repeatedly
- **THEN** each produced notice enters history once with System origin
- **AND** rendering recording time or loading progress does not flood history

### Requirement: Notice lifetimes and owner acknowledgement

Done and Info SHALL last three seconds of visible time, Warn six seconds and manual save success five seconds.
Error SHALL wait for the next new owner press or control movement, without consuming that action.
Releases, MIDI clock, background events and redraws SHALL NOT acknowledge Error.
Startup notices, including the no-key-releases warning, SHALL retain their existing keyboard acknowledgement rules.

#### Scenario: Startup acknowledgement retains its exceptions
- **WHEN** startup warnings or errors appear and keyboard or controller events arrive
- **THEN** general startup notices retain their existing first-key-event acknowledgement
- **AND** keymap warnings keep their separate queue and require a new keyboard press after the warning appeared
- **AND** repeats, releases, modifiers and hidden warnings retain the existing keymap warning behavior
- **AND** each startup result enters history once without changing the effective bindings or read protection

#### Scenario: Timed notices use their visible lifetime
- **WHEN** controlled time reaches a Done, Info, Warn or manual save-success deadline
- **THEN** the notice ends at three, three, six or five seconds respectively and the next notice or normal hint appears

#### Scenario: Hidden notices retain their visible time
- **WHEN** an editor prompt or a higher-priority state hides a timed notice
- **THEN** its visible lifetime pauses until the notice line can show it again
- **AND** new notices still enter history while that line is unavailable

#### Scenario: A delayed update preserves the next notice lifetime
- **WHEN** an update finds an expired notice with another notice pending
- **THEN** the next notice starts its remaining visible lifetime at that update
- **AND** time before that update does not expire a notice that never appeared

#### Scenario: Owner acknowledgement keeps the original action
- **WHEN** an Error is visible and the owner presses a keyboard key, pad or control
- **THEN** that action acknowledges the Error and still follows its normal production route exactly once
- **AND** persistent save state and existing pad-release behavior remain unchanged

#### Scenario: Background and release events cannot dismiss errors
- **WHEN** an Error is visible and note-off, CC release, MIDI clock, loader completion or timer events arrive
- **THEN** the Error stays until a new owner press or control movement
- **AND** required releases and background work still complete normally

### Requirement: Notice priority and bounded queue

Notice priority SHALL be Error, Warn, Done, Info.
A lower level SHALL wait behind a live higher level. An equal level SHALL replace it. A higher level SHALL interrupt it.
Interrupted notices SHALL retain remaining visible time.
Pending storage SHALL hold at most 50 notices and discard the oldest at its lowest level on overflow, never the active Error.
Queue removal SHALL NOT remove an entry from the separate history.

#### Scenario: An informational result cannot hide an error
- **WHEN** an Error is visible and an Info or Done notice arrives without a new owner action
- **THEN** the Error stays visible and the lower-level notice waits
- **AND** both results remain in message history

#### Scenario: Interrupted notices retain remaining time
- **WHEN** a higher-level notice interrupts a timed notice and later ends
- **THEN** the interrupted notice resumes with only its remaining visible time
- **AND** an equal-level replacement does not replay the replaced notice

#### Scenario: Queued notices retain priority and production order
- **WHEN** a notice ends with several notices pending
- **THEN** the highest pending level appears first, with older notices first within that level
- **AND** interruption does not change a notice's original production order

#### Scenario: Expired notices do not resume after interruption
- **WHEN** a higher-level notice arrives exactly at or after the current timed notice's deadline
- **THEN** the expired notice does not enter the pending queue or appear again
- **AND** its history entry remains eligible for the latest 50 entries

#### Scenario: Pending notice overflow remains bounded
- **WHEN** more than 50 pending notices accumulate behind an active Error
- **THEN** the active Error remains and pending storage stays within 50
- **AND** overflow removes the oldest lowest-level pending notice while history retains its own latest 50 entries

### Requirement: F1 message history

F1 SHALL open normal help first and provide a Messages page with the latest 50 notices, newest first, including level and origin.
History SHALL be in memory only and empty after restart.
Messages SHALL remain readable and scrollable at both supported catalogue sizes.
Leaving help SHALL reveal the underlying screen without editing work, dirtying the session or issuing sound commands.

#### Scenario: F1 retains recent results in order
- **WHEN** more than 50 notices arrive and the owner opens F1 Messages
- **THEN** exactly the latest 50 appear newest first with their levels and origins
- **AND** expired and replaced notices remain eligible for that history

#### Scenario: F1 history does not persist or edit work
- **WHEN** the owner reads and closes Messages, then restarts
- **THEN** reading did not change pads, session dirtiness, project or audio commands
- **AND** the new process starts with no previous-process history

### Requirement: Stable F1 window

Help, Messages and Controller SHALL use the same frame across the available body between status, notice and bottom help rows.
At a fixed terminal size, frame position and size SHALL NOT depend on the page, help rows or message count.
All page labels and the shared page/close hints SHALL retain their positions. Only Messages SHALL advertise its scroll controls.
Terminal resizing SHALL adapt all pages to the same body without changing existing message scroll rules or input behavior.

#### Scenario: F1 pages keep stable geometry
- **WHEN** the owner switches between Help and empty or populated Messages at 80×26 or 120×40
- **THEN** both pages keep the same frame corners and dimensions, including after help content or history changes
- **AND** status, notice and bottom help rows remain outside that frame

#### Scenario: F1 controls keep their positions
- **WHEN** the owner switches F1 pages or new messages arrive
- **THEN** both page labels and the Tab/Shift+Tab and F1/Esc hints stay at the same positions
- **AND** the selected page remains distinct while only Messages shows its existing scroll hints

#### Scenario: F1 frame follows terminal size
- **WHEN** the owner enlarges and reduces the terminal with either F1 page open
- **THEN** both pages use the same current body and keep their frame within the terminal
- **AND** Messages retains its existing wrapping, page height and bounded scrolling

#### Scenario: Controller page keeps the F1 frame
- **WHEN** the owner switches to and from the Controller page at 80×26 or 120×40
- **THEN** the frame corners, the page labels and the Tab/Shift+Tab hint stay at the same positions as on Help and Messages
- **AND** the F1/Esc hint stays at its position on Help

### Requirement: F1 history view lifecycle

Each opening SHALL select Help and reset the Messages position. Closing help SHALL retain history.
Redraws SHALL NOT add entries. Long text and origins SHALL remain readable through wrapping and scrolling.
New results SHALL appear newest first while Messages is open.
The view SHALL retain its screen-row offset and clamp it after content or terminal size changes.

#### Scenario: F1 history handles empty and changing content
- **WHEN** the owner opens an empty Messages page, receives new results while scrolled, or resizes the terminal
- **THEN** the page shows an empty-state label or the current history without an invalid scroll position
- **AND** closing and reopening help selects Help and resets the Messages position without removing entries

#### Scenario: Long message text remains readable
- **WHEN** a result or its origin contains long Unicode text at 80×26 or 120×40
- **THEN** the message frame wraps its full text and origin inside its width
- **AND** local scrolling permits reading every screen row without changing the underlying work

### Requirement: Local F1 page controls

Within F1, Tab SHALL select the next page and Shift+Tab the previous page of Help, Messages and Controller.
On Messages, Up/Down and PageUp/PageDown SHALL scroll entries. On Controller, the arrows SHALL select controls. F1/Esc SHALL close help.
These local controls SHALL NOT change global bindings or values behind the overlay.
Existing pad press/release behavior under help SHALL remain.
This change SHALL NOT invent a factory controller binding for history navigation.

#### Scenario: F1 navigation stays inside the overlay
- **WHEN** the owner changes F1 pages or scrolls Messages at either supported size
- **THEN** the visible page and history position change without moving parameters or banks behind help
- **AND** F1/Esc closes help while ordinary controls outside help retain their roles

### Requirement: F1 navigation boundaries

Up/Down SHALL move one screen row. PageUp/PageDown SHALL move one visible page.
Page selection SHALL retain the Messages position until help closes. Scrolling SHALL stop at the content boundaries.
Repeated F1, Esc and page-selection presses SHALL NOT act again. Repeated scrolling MAY continue. Releases SHALL NOT navigate.
Existing input priority and unrelated keyboard and MIDI actions SHALL retain their routes.

#### Scenario: F1 controls retain input priority and bindings
- **WHEN** the owner uses local help keys with text entry, a picker, a question or learning active
- **THEN** the existing higher-priority input state retains those keys and its existing result
- **AND** outside those states a learned help action or an effective keyboard help binding still opens and closes help

#### Scenario: Pad releases survive F1 navigation
- **WHEN** keyboard or raw MIDI holds a pad across opening help, changing its page and closing help
- **THEN** the original release reaches its existing pad and no extra hit or release occurs
- **AND** new pad presses under help retain their existing production route

### Requirement: Truthful question controls

An existing question SHALL show ASK, its current target and effective keyboard answer controls.
A controller answer SHALL appear only for an actual connected binding that reaches that answer through current production dispatch.
Unavailable controller paths SHALL be omitted, not inferred from a template or future `ui.confirm` action.
Question safety, existing answers and non-answer input behavior SHALL stay unchanged.

#### Scenario: Questions show effective working answers
- **WHEN** an existing question opens with default keys, owner key overrides or a working connected controller binding
- **THEN** its answer line names only controls that actually confirm or cancel that question
- **AND** recorded keyboard/raw-MIDI runs through advertised paths retain the existing result and refusal behavior

#### Scenario: Missing controller confirmation stays honest
- **WHEN** a controller has no existing binding that answers the current question
- **THEN** the question shows its keyboard answers without claiming STOP, Apply or another unavailable controller path
- **AND** no new action or binding is created by drawing the answer line

### Requirement: Question answer layout

At both supported terminal sizes, an existing question SHALL retain its target summary, safety warnings and keyboard answer row.
Long target lists SHALL use a visible omitted-row count instead of pushing answers outside the question.
Controller rows SHALL follow the keyboard row. Excess controller rows SHALL use an omitted-row count.
This view SHALL NOT change the stored target or the comparison that asks again when the target changes.

#### Scenario: Question answer rows keep safety visible
- **WHEN** a question has more target or controller rows than fit at 80 by 26 or 120 by 40
- **THEN** its target summary, safety warnings and keyboard answers stay inside the question
- **AND** omitted rows have a visible count
- **AND** the stored confirmation target and its refusal checks stay unchanged

### Requirement: Full help remains readable across pages

F1 SHALL wrap long keys and descriptions without lost text at 80x26 and 120x40 on every current screen.
The current position and effective page keys SHALL remain visible.
Tabs SHALL retain independent offsets. Reopening SHALL start at the first key-table row.
Resize and content changes SHALL clamp to the last full viewport.
The frame's accepted recording caption SHALL remain visible on either tab.
Fixed safety rows, compact-help rows and audio behavior SHALL remain unchanged.

#### Scenario: Every full help row is reachable
- **WHEN** the owner pages through F1 on each current screen at either supported size
- **THEN** long keys, descriptions, recording instructions and cross-bank erasure identities remain readable without lost text
- **AND** the visible page controls follow effective bindings rather than disabled defaults
- **AND** ordinary modifier help does not advertise commands that require the F1 key table

#### Scenario: Full help retains accepted recording feedback
- **WHEN** accepted CountIn, Recording, erasure, Finishing or Idle changes while either F1 tab is open
- **THEN** the visible caption follows that accepted phase and position without a premature queued state
- **AND** the complete target list remains reachable without hiding fixed safety rows

#### Scenario: Help page offsets follow local lifecycle
- **WHEN** the owner changes help tabs, resizes, changes the key-table content or reopens F1
- **THEN** tab offsets remain independent, content changes clamp to a valid viewport and reopening starts at the first key-table row
- **AND** the underlying screen, bank, selection and recording state remain unchanged by page navigation

### Requirement: Bound modifier help includes zoom

Chop Ctrl and Alt help SHALL name effective I/K zoom controls beside their existing J/L bound controls.
F1 SHALL include the effective modified zoom bindings.
Removed or rebound keys SHALL NOT appear as zoom controls.

#### Scenario: Held bound modifiers show working zoom keys
- **WHEN** Chop shows Ctrl, Alt or full help with default or overridden zoom bindings
- **THEN** the help identifies working zoom and bound keys without changing their actions
- **AND** an explicit owner removal or different action changes the shown zoom controls

### Requirement: Pitch Gate help separates settings from sound

Pads help SHALL state that Pitch Gate acts only on A1 notes, not manual hits or previews. On but inapplicable feedback SHALL explain the source or pitch restriction in text. F1 SHALL explain linked slices, positive pitch and the next-grid cut. Toggle popups SHALL retain pad identity, saved state and applicability. Help SHALL use effective keyboard and learned-button controls without new bindings or audio behavior.

#### Scenario: Pitch Gate help explains an inaudible manual toggle
- **WHEN** keyboard or raw MIDI toggles Pitch Gate while the owner plays manual pads with edit open or closed
- **THEN** visible help explains unchanged manual sound and the popup identifies the pad, saved setting and applicability
- **AND** F1 explains A1-only linked-slice positive-pitch cuts with effective controls and the source or pitch reason when inapplicable

### Requirement: Pattern help names the current next step

Count-in help SHALL name cancellation and state that pad hits are not recorded yet. With A1 and no active pass, help SHALL name recording and pass Undo. Recording help SHALL name end, erasure and pass Undo. Erasure help SHALL name silent pad targets and the return to playing. Pattern controls SHALL name selection, value changes and closing. F1 SHALL include actions omitted from compact help. Existing effective-key, modifier and fixed three-row rules SHALL remain.

#### Scenario: Recording help changes with the accepted phase
- **WHEN** count-in, recording, erasure and pattern controls appear in separate fixtures
- **THEN** help names the working next step and effective keys for that phase
- **AND** closing a panel or erasure returns to the applicable previous help without moving screen rows

#### Scenario: Count-in and idle help name hidden rules
- **WHEN** count-in runs, then the pass ends and A1 stays without an active pass
- **THEN** count-in help says that pad hits are not recorded yet, and idle help names recording and pass Undo with effective keys
- **AND** a `none` binding for pass Undo removes that item without a fallback

### Requirement: A completed pass reports its result

When a pass ends, App SHALL post a Done notice with the count of new hits and of replaced older hits. A pass without new hits SHALL say that it added nothing. The notice SHALL NOT replace save or audio warnings.

#### Scenario: A completed pass reports new and replaced hits
- **WHEN** a pass adds three hits, one of them replaces an older hit, and a later pass adds no hits
- **THEN** the first notice reports three new hits and one replaced hit, and the second reports that the pass added nothing
- **AND** save and audio warnings keep their priority over the notice

### Requirement: Pattern refusals explain recovery

Refused recording, length edits, erasure and pattern Undo SHALL name the actual blocking state and a working next step. Manual-save help SHALL state that saving completes the pass while playback continues. Capture overflow and the raw-hit limit SHALL use Warn notices without replacing persistent save or audio status. No-release erasure help SHALL state one loop per pad press.

#### Scenario: Pattern warnings explain limits without claiming success
- **WHEN** a phase, newer Undo step, full pattern or capture overflow prevents an operation
- **THEN** the notice identifies the actual limit and keeps applicable recovery controls visible
- **AND** it does not show a success result or hide save and audio warnings

#### Scenario: Save and erase fallbacks are stated before use
- **WHEN** F1 shows pattern recording help with or without key-release support
- **THEN** it explains pass completion on manual save and the applicable erase hold or one-loop fallback
- **AND** microphone hold or toggle remains a separate instruction

### Requirement: Pattern view help names its keys and marks

When A1 exists, ordinary Pads help SHALL show the effective view keys. F1 SHALL explain the onset, between-line and multiple-hit marks, the playback plate, the all-pads row and the underlined banks. Owner overrides and `none` SHALL stay effective in help.

#### Scenario: Pattern view help names view keys and marks
- **WHEN** Pads shows A1 with default or owner-moved view keys, and the owner opens F1
- **THEN** the help line shows the effective view keys, and F1 explains each mark of the strip and the bank row
- **AND** a `none` binding removes the key from help without an invented fallback

### Requirement: Pattern tidy help and results

Pattern controls SHALL show a Clear A1 row with the count of A1 hits and the key that runs it. An empty A1 SHALL show the row dim. A result SHALL post a Done notice with its count, for example `A1: pad Q kick erased, 5 hits. Ctrl+Z restores.` or `A1 cleared: 12 hits.`. A command that changes nothing SHALL post an Info notice.

#### Scenario: Pattern tidy results name their counts
- **WHEN** the owner erases a pad with hits, clears A1, and then erases a pad without hits
- **THEN** each change posts a Done notice with its count, and the empty erase posts an Info notice
- **AND** pattern controls name the effective Clear A1 key

### Requirement: Erasure help names both forms

In whole-pad erasure, the help line SHALL say that a pad erases all its A1 hits. It SHALL name Undo and the exit keys. F1 SHALL explain both erasure forms: under the playhead in a pass, whole pads outside a pass.

#### Scenario: Erasure help names both forms
- **WHEN** the owner turns on erasure on Pads outside a pass and opens F1
- **THEN** the help line names the whole-pad erasure, Ctrl+Z and Esc
- **AND** F1 explains held erasure in a pass and whole-pad erasure outside a pass

#### Scenario: Erasure outside Pads names Pads
- **WHEN** no pass is active, the owner is on Chop and presses Alt+E
- **THEN** erasure stays off and an Info notice says to erase pads on Pads (F2)

### Requirement: Quantize help names the next hits

When the owner selects Quantize or Strength in pattern controls, the row hint SHALL start with `next hits`. Strength SHALL keep its step after it: `next hits, step 10`. F1 SHALL say that Quantize and Strength set the next recorded hits and do not move saved hits.

#### Scenario: Quantize help names the next hits
- **WHEN** the owner selects Quantize, then Strength in pattern controls, then opens F1
- **THEN** each selected row shows `next hits`
- **AND** F1 says that a change does not move saved hits

### Requirement: Counts and undo name what they mean

A count of one hit SHALL say `1 hit`. Other counts SHALL say `N hits`. This SHALL apply to pad counts, notices and questions. The undo notice of a pattern step SHALL name that step: `erase pad <key>`, `clear` or `pattern pass`.

#### Scenario: Counts and undo name what they mean
- **WHEN** the owner erases a pad with one hit, presses Ctrl+Z, clears A1 and presses Ctrl+Z
- **THEN** the pad count and notice say `1 hit`, and the undo notices say `undo: erase pad W on A1` and `undo: clear on A1`

### Requirement: The pad edit help comes first

While the pad edit is open, the help row SHALL show the edit group before the A1 group.
If the A1 group does not fit, it SHALL drop whole, as other groups do. F1 SHALL show it.

#### Scenario: The pad edit help comes first
- **WHEN** the owner opens the pad edit at 80x26
- **THEN** the help row shows the edit keys before the A1 keys
- **AND** the A1 group drops whole when it does not fit, and F1 shows it

### Requirement: F1 wraps whole words

F1 SHALL wrap descriptions and keys at spaces. A word wider than its column SHALL wrap inside the word.

#### Scenario: F1 wraps whole words
- **WHEN** the owner opens F1 at 80x26
- **THEN** no line ends inside a word that fits its column

### Requirement: F1 controller tab

F1 SHALL have a `Controller` tab after `Messages`. It SHALL draw the `[layout]` of one connected device inside the F1 frame.
Each control SHALL show a label of not more than 6 cells. The label SHALL be the action that the control does now on the screen under F1, with the open edit layer and the learned bindings.
A free control SHALL show `-`. A binding to an action that the program does not have does nothing, so it SHALL show `-` too. Each control kind SHALL have its own brackets.

#### Scenario: Controller tab draws the device top
- **WHEN** the X-Session Pro is connected and the owner opens the F1 `Controller` tab at 80x26 or 120x40
- **THEN** the tab draws each control of its layout at its place with its kind
- **AND** each label is the short label of the action that the same control does through the MIDI dispatch

#### Scenario: Controller labels follow the screen and the edit
- **WHEN** the owner opens the `Controller` tab on F2, on F3, and on F2 with the pad edit open
- **THEN** each label shows the action of that screen and layer
- **AND** a learned binding shows over the profile action, and a free control shows `-`

#### Scenario: Every action has a short label
- **WHEN** the program lists all its actions
- **THEN** each action has an ASCII short label of 1 to 6 cells

### Requirement: Controller tab selection

On the `Controller` tab, the arrows SHALL select the nearest control in their direction. Empty places SHALL be skipped.
A line under the drawing SHALL give the control name, its kind, its action and the action's full label.
The selection SHALL NOT send an action or change a value. Pad presses and releases under F1 SHALL keep their route.

#### Scenario: Arrows select controls on the drawing
- **WHEN** the owner moves the selection with the arrows across the X-Session layout
- **THEN** the selection skips empty places and stops at the edges
- **AND** the line under the drawing names the selected control and its action

#### Scenario: Selection does not act
- **WHEN** the owner moves the selection while a controller pad is held
- **THEN** no action, value change or sound command comes from the selection
- **AND** the pad release reaches its pad

### Requirement: Controller device pages

The `Controller` tab SHALL show one device. A top line SHALL name each connected device and mark the current one.
The ports of one ALSA client with one profile SHALL be one device. Two devices of one profile SHALL show as `<name> 1` and `<name> 2` in the order of connection.
On `Controller`, the help page actions SHALL change the device. The tab SHALL start on the device of the last MIDI event, or on the first connected device.

#### Scenario: Help page keys change the device
- **WHEN** two devices are connected and the owner presses Alt+PgDn on the `Controller` tab
- **THEN** the tab shows the other device and the top line marks it
- **AND** Alt+PgDn on the `Help` tab still pages the key table

#### Scenario: Tab starts on the last used device
- **WHEN** the owner turns a knob on the second device and then opens the `Controller` tab
- **THEN** the tab shows the second device

#### Scenario: Two devices of one profile have numbers
- **WHEN** two devices with the same profile are connected
- **THEN** the top line names them with the numbers 1 and 2 in the order of connection

#### Scenario: Ports of one device share a page
- **WHEN** the X-Session Pro connects with its ports `MIDI 1` and `MIDI 2`
- **THEN** the top line names one X-Session Pro without a number

#### Scenario: Device disconnects while its page is open
- **WHEN** the shown device disconnects while the `Controller` tab is open
- **THEN** the tab shows the next connected device, or the empty state when no device is left
- **AND** a device that connects later gets a place on the top line without a change of the current page

### Requirement: Controller tab without a drawing

A device without a valid layout that fits the frame SHALL show a list of its controls and their actions. Up and Down SHALL move the selection, and the list SHALL scroll with it.
Without a connected device, the tab SHALL show `No MIDI device is connected.` and the names of the loaded profiles.

#### Scenario: Profile without a layout shows a list
- **WHEN** a device without `[layout]` is connected and the owner opens the `Controller` tab
- **THEN** the tab lists each control with its action, and the selection scrolls the list

#### Scenario: Layout too wide for the frame shows a list
- **WHEN** a layout row has more places than the frame width allows
- **THEN** the tab shows the list for that device

#### Scenario: No device shows the known profiles
- **WHEN** no MIDI device is connected and the owner opens the `Controller` tab
- **THEN** the tab shows `No MIDI device is connected.` and the names of the loaded profiles

### Requirement: A1 overview help explains its live controls and marks

The overview help band SHALL show effective F2, bank, view, and empty-row controls. F1 SHALL explain event symbols, velocity levels, the playback plate, bank sections, and the filtered or full table. An absent A1 SHALL give the effective recording action. Owner overrides and `none` SHALL change the shown keys. Existing notice priority SHALL keep save and audio warnings visible.

#### Scenario: Overview help follows effective bindings
- **WHEN** the owner opens the overview with default keys, moved keys, or explicit `none`, then opens F1
- **THEN** the help band and F1 name only effective actions and explain event marks, velocity, and row scope
- **AND** the help returns to the same overview without changing A1 or hiding a safety warning

### Requirement: F5 names the pattern destination

Short help on other screens SHALL name the F5 destination `pattern`. The overview title SHALL name `PATTERN` and `A1`. A short label SHALL NOT use `A1` alone for this screen action.

#### Scenario: Pattern key names its destination
- **WHEN** the owner reads short help on Files and opens the overview with F5
- **THEN** short help names `pattern` and the overview title names `PATTERN` and `A1`

### Requirement: A1 correction help names its active controls

The overview help SHALL show the effective F7 entry key. In correction it SHALL state that J/L or learned MIDI step controls choose a time step. It SHALL state that a pad press adds or removes that pad's hit at this step. It SHALL name event choice, view, bank, Undo, and exit controls. F1 SHALL explain the time column, selected pad row, audio plate, multiple-event choice, and silent insertion. Owner overrides and `none` SHALL change the shown keys.

#### Scenario: Correction help follows the next action
- **WHEN** the owner opens the overview, enters correction, selects empty and multiple-event cells, then opens F1
- **THEN** help names the applicable addition, selection, removal, or exit action with effective keys
- **AND** it does not describe an edit pad press as live playing

#### Scenario: Correction help separates step from pad target
- **WHEN** the owner moves the time cursor and presses a different pad at that step
- **THEN** help and F1 identify keyboard or learned MIDI step controls as time selection and pad keys as row selection and hit toggle
- **AND** the header and result use the same step number

### Requirement: A1 correction feedback states one-hit scope

Only a successful addition, removal, or Undo SHALL report a changed event. The result SHALL name its bank, pad, step, and one-hit scope and direct the owner to Undo. The help band SHALL show the effective Undo key, including `none`. A refusal SHALL name absent A1, a conflicting owner, full capacity, or stale event as applicable and give a safe next action. Save and audio warnings SHALL keep their existing priority.

#### Scenario: Correction feedback separates focus from edits
- **WHEN** an empty-cell addition, multiple-event focus, removal, Undo, and stale selection occur in order
- **THEN** only addition, removal, and Undo claim a changed hit, and each names one-event scope
- **AND** a refusal or F1 preserves the edit target and existing safety warnings

#### Scenario: Correction feedback separates focus from removal
- **WHEN** a multiple-event focus, removal, Undo, and stale selection occur in order
- **THEN** only removal and Undo claim a changed hit, and each names one-event scope
- **AND** a refusal or F1 preserves the edit target and existing safety warnings

### Requirement: Pattern help names its action

The Pads help group for A1 SHALL say `PATTERN`. The action that opens pattern controls SHALL use the visible text `pattern keys` with its effective binding. With the default binding, it SHALL say `Alt+N: pattern keys`. A missing or disabled binding SHALL NOT show a false key. MIDI learn SHALL show `pattern` for the existing pattern screen action without changing its saved action ID or group.

#### Scenario: Pattern keys follow the effective binding
- **WHEN** the owner opens Pads and F1 with the default, moved, or disabled pattern-controls binding
- **THEN** the help group says `PATTERN` and names only the effective key for `pattern keys`
- **AND** MIDI learn shows `pattern` while old saved assignments still work
