## MODIFIED Requirements

### Requirement: Screen columns and titles

The right column SHALL start at the bank row.
F2 SHALL show meters and pad panel. F6 SHALL show its path and filter above the file list. F3 SHALL show slice information.
The title SHALL stand above the strip, naming the held file or error on F6 and file and slices on F3.
Existing fixed screen/playback badges SHALL retain their widths and playback rules. Other mode indicators SHALL use shared title badges.

#### Scenario: The folder above its list
- **WHEN** the files screen is open
- **THEN** the folder path stands above its first list row in the right column
- **AND** mode badges do not move the strip, grid, bank row or right column

## ADDED Requirements

### Requirement: Shared title mode badges

Every screen SHALL use one shared composition for existing mode badges.
Badges SHALL have text and a semantic color: yellow for PadMode, cyan for On and magenta for Attention.
They SHALL show at most three optional badges, separate from fixed playback slots.
An active PadMode SHALL come first, then ASK, recording and ordinary On badges in that order.
Color alone SHALL NOT identify a mode.

#### Scenario: Mode badges follow the active screen state
- **WHEN** keyboard or raw MIDI actions enter and leave existing modes on Pads, Files, Chop, FX and Settings
- **THEN** their title badges follow the actual current state on the next frame
- **AND** leaving a mode removes its badge without moving shared screen rows

#### Scenario: Badges retain identity without color
- **WHEN** title badges appear in a terminal with limited colors
- **THEN** their text still distinguishes mode, editing, learning, questions and recording

### Requirement: Existing mode badge labels

The title SHALL name active placement modes `IN HAND`, `COPY`, `SWAP` or `ASSIGN`.
It SHALL name marking `MARK` or `N MARKED`, editing `EDIT` with its page, learning `LEARN` and questions `ASK`.
Recording SHALL show `REC` with elapsed time when space permits. Chop toggles SHALL show `LOOP` and `AS PAD`.
Mark and assignment badges SHALL retain their counts. Inactive states SHALL NOT produce these badges.

#### Scenario: Every existing mode has its badge
- **WHEN** each existing placement, marking, edit, learning, question, recording or Chop toggle state is active in a separate fixture
- **THEN** its badge uses the specified label, page, count or elapsed time
- **AND** no badge advertises transport, pattern or controller-panel state

#### Scenario: Overlapping placements remain visible
- **WHEN** more than one of file placement, copy and swap is pending
- **THEN** one compound PadMode badge names the pending placement modes
- **AND** existing keyboard and MIDI action priorities remain unchanged

### Requirement: Badge width and compatibility

At supported widths, badges SHALL retain screen identity and an active PadMode label.
Lower-priority optional badges SHALL disappear first, then remaining labels SHALL use compact forms without losing mode identity.
PLAY and LISTEN SHALL retain their existing ten-cell slots, styles, actual-playback semantics and stable following columns as playback changes.
Badge changes SHALL NOT change sound, effective keys, saved data or input order.

#### Scenario: Narrow titles retain the pad mode
- **WHEN** several badges compete for title width at 80×26
- **THEN** the screen and current pad mode remain readable, with lower-priority badges omitted
- **AND** no text crosses the title boundary

#### Scenario: Playback slots remain stable beside modes
- **WHEN** PLAY or LISTEN changes between actual playback and silence with optional badges present
- **THEN** each playback slot retains its ten-cell width and existing active or idle style
- **AND** following title content stays in the same columns during that playback change

### Requirement: Shared value popup

An applied parameter change SHALL show its object, parameter and formatted value for one second, with a range bar for numeric values.
The popup SHALL use the strip's top right or the corresponding content area below the title on screens without a strip.
It SHALL NOT move rows or hide editor prompts or questions.
Local highlights and limited-pad counts SHALL remain. Pickup waiting or refusal SHALL NOT show an unapplied value.

#### Scenario: Values appear on every screen
- **WHEN** keyboard or raw MIDI changes an applicable value on each current screen
- **THEN** the shared popup names the actual object, parameter and applied value for one second
- **AND** its expiry restores the underlying content without another input

#### Scenario: Refused and limited values remain truthful
- **WHEN** pickup waits, the system refuses a change, or a scope contains pads at their limits
- **THEN** no refused value appears as applied
- **AND** applied scope changes retain the existing count of limited pads

### Requirement: Value popup deadlines

Only an actual value change SHALL start or restart the one-second deadline.
The deadline SHALL use elapsed time, including time behind a higher-priority overlay.

#### Scenario: Value deadlines follow actual changes
- **WHEN** repeated equal values or a limit prevent another change after an applied change
- **THEN** the original popup expires exactly one second after that applied change
- **AND** a later actual change starts a new one-second deadline

### Requirement: Value popup object identity

Object labels SHALL identify the original pad and bank, slice, shared FX slot, master or Settings choice.
A pad-scope popup SHALL name one actual pad value, not imply that all marked pads have that value.
Buffer choices SHALL identify the configured request. Wave choices SHALL state that they apply on the next start.
The popup SHALL disappear if its recorded value or object changes, or a project replaces the current session.

#### Scenario: Value snapshots retain their original object
- **WHEN** the owner selects another pad or screen while a popup remains active
- **THEN** the popup still names the original object and its applied value
- **AND** undo, object replacement or project replacement cannot leave an obsolete value visible

### Requirement: Value popup priority

Help, text entry, pickers, questions and learning SHALL hide the popup without changing their input routes.
Existing notices and local highlights SHALL retain their behavior.

#### Scenario: Editor prompts take priority over values
- **WHEN** Help, text entry, a picker, a question or learning covers an active value popup
- **THEN** the popup does not cover that interface
- **AND** closing that interface after the value deadline does not restore the expired popup

### Requirement: Shared status segments

The status SHALL show persistent save state, recording, input, audio and project in that order.
It SHALL list KB and actual connected devices, with distinguishable compact labels.
A new owner input SHALL highlight its actual source for one second.
Missing origin data SHALL NOT become a guessed device.
No transport or layer indicator SHALL appear before its corresponding feature exists.

#### Scenario: Status follows keyboard and connected inputs
- **WHEN** keyboard and two connected MIDI devices produce events in turn
- **THEN** the input segment lists the connected devices and highlights the actual most recent source
- **AND** source metadata does not change velocity, action results, audio routing or release ownership

#### Scenario: Disconnect cannot leave a false active device
- **WHEN** a device disconnects and a delayed event from its old connection arrives
- **THEN** the input list does not show that old connection as connected or active
- **AND** a reconnect gets a current connection identity without inheriting a stale highlight

### Requirement: Status width priorities

At supported widths, the status SHALL omit project first, then audio detail, then input detail.
It SHALL keep recording, persistent save warnings, audio-loss warnings, nonzero xruns and the no-key-releases marker visible.
Long device or project names SHALL NOT cover these indicators or escape the status row.
Omitted input details SHALL NOT remove message-history origin information.

#### Scenario: Narrow status retains safety indicators
- **WHEN** recording, a failed save, audio warnings and long input/project labels compete at 80×26
- **THEN** optional details shorten or disappear while recording and applicable safety indicators remain readable
- **AND** the help band and shared screen rows do not move

### Requirement: Overlay image safety

A text overlay SHALL remain visible with kitty graphics enabled or disabled.
An intersecting image SHALL be hidden or removed before the text draws and restored after the overlay ends.
Dismissal, expiry, resize and screen changes SHALL leave no stale overlay or missing wave.
Text prompts and questions SHALL retain precedence over value popups.

#### Scenario: A value overlay restores its wave
- **WHEN** a value popup covers a kitty wave and expires without further input
- **THEN** graphics commands hide the intersecting image before the text and restore it after expiry
- **AND** graphics-disabled rendering shows equivalent readable text without kitty commands

#### Scenario: Resize and questions keep overlays readable
- **WHEN** a question opens or the terminal resizes while a value popup is active
- **THEN** text remains inside its area, the question stays readable and the underlying wave restores when appropriate

### Requirement: Visual timer progress

Unchanged badge and status content SHALL avoid repeated reconstruction.
Caching SHALL NOT stop notice expiry, value expiry, input highlights, recording elapsed time, level meters or existing playback indicators.
The UI SHALL update those states when their inputs or deadlines change without requiring another owner action.
This optimization SHALL NOT change audio-thread work or input ordering.

#### Scenario: Unchanged views do not freeze timed feedback
- **WHEN** no new owner action arrives while recording, meters, values or notices remain active
- **THEN** elapsed time and levels advance, expired feedback ends and the UI reuses unchanged static content
- **AND** current playback indicators still follow real voices

### Requirement: Offline screen catalogue

The system SHALL provide `task screens` for deterministic ANSI screen files under `renders/screens/` at 120×40 and 80×26.
Names SHALL use `<screen>-<mode>-<width>x<height>.ansi`.
A checked matrix SHALL cover every registered screen and applicable current mode. Missing entries SHALL fail a test.
Generation SHALL NOT open audio or MIDI devices, change projects or require new dependencies.

#### Scenario: Catalogue covers screens and existing modes
- **WHEN** the offline catalogue generator runs
- **THEN** both sizes contain every registered screen and its applicable normal, edit, marking, placement, learning, question, recording and message states
- **AND** the catalogue includes F1 Help and Messages without claiming a hardware graphics pass

#### Scenario: Missing catalogue entries fail coverage
- **WHEN** a required screen/mode entry or supported size is absent from the catalogue matrix
- **THEN** its coverage test fails and names the missing entry
- **AND** ordinary catalogue generation does not change configuration or session files
