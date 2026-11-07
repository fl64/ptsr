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

Help and Messages SHALL use the same frame across the available body between status, notice and bottom help rows.
At a fixed terminal size, frame position and size SHALL NOT depend on the page, help rows or message count.
Both page labels and the shared page/close hints SHALL retain their positions. Only Messages SHALL advertise its scroll controls.
Terminal resizing SHALL adapt both pages to the same body without changing existing message scroll rules or input behavior.

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

Within F1, Tab/Shift+Tab SHALL select Help or Messages.
On Messages, Up/Down and PageUp/PageDown SHALL scroll entries. F1/Esc SHALL close help.
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
