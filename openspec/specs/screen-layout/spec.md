# screen-layout Specification

## Purpose
Use a shared frame for Pads, Files and Chop. Keep pad visibility, waveform space and startup loading information consistent as the terminal size changes.

## Requirements

### Requirement: One layout for the screens with pads
Pads (F2), Files (F6) and Chop (F3) SHALL share strip and grid size and position: wave above, pads left, mode column right.
The full-width strip SHALL show live output on F2, the selected file on F6, and the sample with cuts on F3.
The bank row SHALL stand directly above the grid on each screen.

#### Scenario: The same frame on every screen
- **WHEN** the owner switches F2 → F6 → F3 in the same terminal
- **THEN** the strip and the pad grid stay at the same rows and columns, and the pads look the same, waves included; only the right column and the content of the strip change

### Requirement: Screen columns and titles

The right column SHALL start at the bank row.
F2 SHALL show meters and pad panel. F6 SHALL show its path and filter above the file list. F3 SHALL show slice information.
The title SHALL stand above the strip, naming the held file or error on F6 and file and slices on F3.
Existing fixed screen/playback badges SHALL retain their widths and playback rules. Other mode indicators SHALL use shared title badges.

#### Scenario: The folder above its list
- **WHEN** the files screen is open
- **THEN** the folder path stands above its first list row in the right column
- **AND** mode badges do not move the strip, grid, bank row or right column

### Requirement: Wave strip height

The strip SHALL take all height left by the grid and pad waves, at least 8 rows.
When space is short, pad waves SHALL shrink first, then the strip down to 3 rows, so a 26-row terminal still fits the whole grid.

#### Scenario: A tall terminal gives the strip the rest
- **WHEN** the terminal is 60 rows high
- **THEN** the pads draw their waves in three rows, the strip takes all the rows left above them, more than 8, and is the same height on F2, F6 and F3

#### Scenario: A low terminal
- **WHEN** the terminal gets lower step by step
- **THEN** the pad waves take fewer rows first, then the strip gets lower down to 3 rows, and the whole grid always fits

### Requirement: Hide the pads
`Ctrl+O` SHALL toggle the pad grid on F2, F6 and F3, with one persistent state shared across restarts.
When hidden, the strip SHALL replace the grid in the left column from the title line down, with one pad line below.
The right column SHALL retain its full height from the title line without truncation.
Chop assign SHALL show the grid while open.

#### Scenario: Hide the pads
- **WHEN** the owner presses `Ctrl+O` on the pads screen
- **THEN** the grid is gone, the strip is taller by the rows of the grid, and the pad line shows the bank row with bank 1 in brackets and then `pad W kick` for the selected pad W

### Requirement: Hidden-grid pad line

The pad line SHALL show every bank with the same fill as above the grid, the open bank in brackets, then the selected pad's name.
It SHALL flash on a hit.
While open-bank pads are held or flash, that bank's glyphs SHALL show only those pads, in white, then return to the fill.
A narrow window SHALL truncate the pad part first.

#### Scenario: Every bank in the pad line
- **WHEN** the pads are hidden, bank 1 is open and bank 3 holds sounds
- **THEN** the pad line shows every bank with its number, bank 3 with its fill and bank 1
  in brackets, each in its color, the same as the bank row above the grid

#### Scenario: A hit lights in the bank row
- **WHEN** the pads are hidden and the owner holds pad Q of bank 3
- **THEN** while pad Q is held or flashes, the glyphs of bank 3 in the bank row show only
  pad Q, in white, no other pad of the bank; after that they show the fill again

#### Scenario: The right column keeps its rows
- **WHEN** the pads are hidden
- **THEN** the right column shows all of it as with the pads shown: the L and R meters and the overload light with the pad panel on F2, the path and the list on F6, the slice line and the side list on F3

#### Scenario: The same on every screen
- **WHEN** the pads are hidden and the owner switches to F6 and F3
- **THEN** both screens draw without the grid, with the same pad line and the same strip height

#### Scenario: The pad line flashes
- **WHEN** the pads are hidden and the owner hits pad Q
- **THEN** the pad line names pad Q and flashes like a hit pad

#### Scenario: The plan shows the grid
- **WHEN** the pads are hidden and the owner opens the assign mode on the chop screen
- **THEN** the grid is drawn with the plan while the assign mode is open, and is hidden again when it closes

#### Scenario: The choice survives a restart
- **WHEN** the owner hides the pads and restarts the program
- **THEN** the pads are hidden; `Ctrl+O` shows them, and a restart keeps them shown

### Requirement: The splash in the wave strip
At start, the master wave strip above the pad grid SHALL show a splash instead of the silent wave until the first sound.
It SHALL show `ptsr - PT's SampleR`, build version, loading line and this exact logo:

```
           
  _ _/_ __
 /_// _\/ 
/         
```

With fewer than four strip rows, the splash SHALL show `ptsr` and the loading line instead of full logo, name and version.

#### Scenario: The splash at start
- **WHEN** the owner starts ptsr
- **THEN** the wave strip shows the logo, `ptsr - PT's SampleR`, the version and the
  loading line, and the pads, the help line and the status line show as always

### Requirement: First sound ends the splash

The first sounding keyboard/controller pad note, sample preview or started recording SHALL end the splash permanently and show the wave.
A sound request SHALL end it only when the audio queue accepts it while the project is ready.
No audio, a full queue, an unready project or a silent pad hit SHALL NOT end it.
Keys or elapsed time alone SHALL NOT close it.

#### Scenario: A pad hit during the splash
- **WHEN** the splash shows and the owner hits pad `Q`
- **THEN** pad `Q` plays at once and the strip shows the wave from then on

### Requirement: Splash screen and edit scope

The splash SHALL appear only on Pads, with other screens showing their own content.
Returning to Pads before the first sound SHALL show it again.
Pad edit SHALL replace it with its envelope or filter curve.
Closing edit before the first sound SHALL restore the splash.

#### Scenario: A command key during the splash
- **WHEN** the splash shows and the owner presses F3
- **THEN** the chop screen shows its own content; when the owner returns to the pads screen
  before the first sound, the splash shows again

#### Scenario: The pad edit takes the strip from the splash
- **WHEN** the owner opens the pad edit before the first sound
- **THEN** the strip shows the envelope or filter curve instead of the splash
- **AND WHEN** the owner closes the edit before the first sound
- **THEN** the splash shows again

### Requirement: Splash loading status

The session SHALL load before the first screen.
The loading line SHALL count pads with samples loading, say `ready` after loading, or name a file that failed to load.
Sample loading SHALL continue under the splash, and the rest of the screen SHALL remain as usual.

#### Scenario: A missing sample on the splash
- **WHEN** the session names a sample file that does not exist and the owner starts ptsr
- **THEN** the loading line names that file

### Requirement: Persistent save status

Every screen SHALL keep one `*` instead of `Unsaved`: dim when saved, yellow when pending. Only its color SHALL change.
It SHALL follow the project name or stay separate if that name is absent or omitted.
Write refusal SHALL add `Save failed` with existing warning style and width priority until saving succeeds.
Notices SHALL NOT hide either indicator. Saving SHALL dim `*` and remove `Save failed` only after success.
Save timing, retries and stored data SHALL stay unchanged.

#### Scenario: Pending work has a visible save state
- **WHEN** a keyboard or controller action changes the session before autosave
- **THEN** the status shows a yellow `*` on Pads, Files, Chop, FX and Settings instead of `Unsaved`

#### Scenario: A save failure stays visible after its notice
- **WHEN** a save fails and its notice expires or another notice appears
- **THEN** `Save failed` remains visible in the shared status line on every screen, including small and large supported terminals
- **AND** no new mode opens, no row moves, and existing help, notices and controls remain available

#### Scenario: Successful retry clears the save warning
- **WHEN** a manual retry saves all pending work
- **THEN** `*` becomes dim and `Save failed` disappears from the status
- **AND** a later edit makes `*` yellow until its save succeeds

#### Scenario: Pending work uses a quiet project marker
- **WHEN** save state changes between saved and pending with a visible, absent or width-omitted project name at either supported size
- **THEN** one `*` changes between dim and yellow without moving status columns or showing `Unsaved`
- **AND** a save failure keeps the marker and adds the full warning until saving succeeds

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

The status SHALL show save failures, recording, input, audio and project in that order.
Recording SHALL show seconds when space permits.
Input SHALL keep `KB` and the last used connected controller number, such as `M8`.
Only the newest source SHALL have emphasis for one second after receipt. Expiry SHALL retain both labels.
Missing origin SHALL NOT become a guessed device. No transport or layer indicator SHALL appear before its feature exists.

#### Scenario: Status follows keyboard and connected inputs
- **WHEN** keyboard and two connected MIDI devices produce events in turn
- **THEN** the input segment keeps `KB` and the last used controller number, with emphasis on the newest input during its original one-second deadline
- **AND** source metadata does not change velocity, action results, audio routing or release ownership

#### Scenario: Disconnect cannot leave a false active device
- **WHEN** a device disconnects and a delayed event from its old connection arrives
- **THEN** the input list does not show that old connection as connected or active
- **AND** a reconnect gets a current connection identity without inheriting a stale highlight

### Requirement: Last controller identity

The number SHALL use existing connected-source numbering, without a new physical-device grouping inferred from port names.
Only actual controller input SHALL select it. Keyboard input, releases, equal values, derived notifications and stale deliveries SHALL NOT select another number.
The controller field SHALL NOT list full port names, other controllers or a count.
Full port names and input events SHALL remain available in F12 Input.

#### Scenario: Only the last used controller number stays visible
- **WHEN** M8 produces new input among ten connected controllers, its highlight expires and the owner uses the keyboard
- **THEN** KB and M8 remain in the input segment without a controller list or count
- **AND** later input from M10 replaces it with M10, while releases and stale events do not select another number

### Requirement: Last controller connection lifetime

Before the first controller input or after that controller disconnects, the segment SHALL show no controller number.
Reconnection SHALL NOT inherit a disconnected connection's last-used state.
Missing metadata SHALL NOT select or invent a number.

#### Scenario: Last controller number requires current connection input
- **WHEN** controllers connect before any input, then one produces input, disconnects and reconnects
- **THEN** the number appears only after current-connection input and disappears on its disconnection
- **AND** reconnection and late input from the old connection do not restore the old last-used state

### Requirement: Stable controller segment width

Input SHALL reserve `KB`, one marker per source and a field for the widest current controller number.
With an unchanged connected set and terminal width, keyboard activity, number changes and expiry SHALL NOT move following status columns.
Connection changes and terminal resizing MAY recompute that width. Safety indicators SHALL retain their existing priority.

#### Scenario: Controller activity keeps status columns fixed
- **WHEN** input alternates between M1, M8 and M10 among ten connected controllers and their highlights expire
- **THEN** the reserved controller segment width and following audio and project columns stay unchanged at both supported terminal sizes
- **AND** marker changes and number changes do not expand the segment or remove safety indicators

### Requirement: Permanent keyboard identity

`KB` SHALL remain visible before any input, without connected controllers and after its highlight expires, subject to existing safety width priorities.
Keyboard repeats and releases SHALL NOT extend its highlight.
Keyboard activity SHALL NOT restart the retained controller's deadline.

#### Scenario: Keyboard stays visible without controllers
- **WHEN** no controller is connected and a keyboard press, repeat and release occur
- **THEN** `KB` remains visible before, during and after its one-second highlight
- **AND** repeats and releases do not extend the original deadline or move following status columns

### Requirement: Input source identity

Source metadata SHALL retain the actual port, connection identity and original receipt time through direct and deferred MIDI routes.
A new connection SHALL have a new identity. Missing metadata SHALL NOT establish a connection or guess a source.
These rules SHALL NOT change action dispatch, notice acknowledgement, project epochs, velocities or release ownership.

#### Scenario: First input can precede the connection notice
- **WHEN** a connection produces input before its connected notification reaches the source model
- **THEN** that input stays pending until the matching connection notification arrives
- **AND** only its original, unexpired deadline can highlight the newly listed connection

#### Scenario: Missing source metadata stays unknown
- **WHEN** a routed event has no connection or origin metadata
- **THEN** it cannot establish a connected-device entry or highlight an inferred device
- **AND** its existing action, sound and release behavior stays unchanged

### Requirement: Input source highlight lifetime

Only a new keyboard press or MIDI owner input SHALL start the one-second highlight.
Releases, keyboard repeats, equal knob values and derived notifications SHALL NOT restart it.
Queued delivery SHALL NOT extend the deadline or replace a source with an older input.
Project replacement SHALL NOT rename input sources or restart their deadlines.

#### Scenario: Input highlights retain original receipt time
- **WHEN** direct or deferred MIDI notifications arrive after another source produced newer input
- **THEN** the source highlight still follows the newest original input and expires one second after receipt
- **AND** releases, repeats and derived notifications do not extend that deadline

### Requirement: Input source continuous values

Continuous values SHALL compare within their own connection, channel and control.
Old connections and older control values SHALL NOT replace the current comparison.
The first value in a new connection SHALL count as movement, even if an old connection sent the same value.

#### Scenario: Source knob comparison does not inherit an old connection
- **WHEN** a late old connection sends the same knob value before the current connection's first value
- **THEN** only the current connection's value starts its highlight
- **AND** the old event does not seed its comparison

#### Scenario: Source knob comparison rejects older values per channel and control
- **WHEN** an older control value arrives after a newer value and the owner repeats the newer value
- **THEN** neither delivery restarts the highlight
- **AND** equal values on different channels or controls remain independent

### Requirement: Compact audio status

The permanent status SHALL use actual backend, buffer size, sample rate, latency, master volume and project values.
It SHALL NOT show CPU percentage or zero xruns. CPU diagnostics SHALL remain available in Settings.
Nonzero xruns and audio loss SHALL retain their warnings and width priority.

#### Scenario: Compact status retains actual audio values
- **WHEN** the audio format, backend, master or project changes at either supported size
- **THEN** the status shows the current values without permanent CPU or zero-xrun fields
- **AND** nonzero xruns, audio loss and Settings CPU diagnostics remain available

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

### Requirement: Assignment target text bounds

At supported terminal sizes, the assignment sidebar SHALL wrap target names by terminal-cell width.
A wide grapheme SHALL move to the next row if the remaining row has too few cells. A long word SHALL split only between graphemes.
This formatting SHALL NOT change assignment targets, confirmations or input routing.

#### Scenario: Long assignment target names stay in the sidebar
- **WHEN** an assignment target name contains wide or combining graphemes near the sidebar's right edge
- **THEN** every visible grapheme fits inside its row at 80 by 26 and 120 by 40
- **AND** the target order, pad names and assignment actions stay unchanged

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
