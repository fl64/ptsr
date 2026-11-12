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

The first sounding keyboard/controller pad note, sequenced pad note, sample preview or started recording SHALL end the splash permanently and show the wave. A manual request SHALL end it only when the audio queue accepts it while the project is ready. Rejected requests, absent audio, unready projects and silent events SHALL NOT end it. Keys or elapsed time alone SHALL NOT close it.

#### Scenario: A pad hit during the splash
- **WHEN** the splash shows and the owner hits pad `Q`
- **THEN** pad `Q` plays at once and the strip shows the wave from then on

#### Scenario: Sounding pattern notes end the splash without selecting a pad
- **WHEN** an accepted current-project pattern note starts while the splash shows
- **THEN** the strip changes to live output without changing the selected pad or input-source identity
- **AND** an empty pattern, silent event or unready project does not close the splash

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

Every screen SHALL use shared title composition. Badge colors SHALL remain yellow for PadMode, cyan for On and magenta for Attention. Transport SHALL share the first mode badge, or occupy the first badge when no mode is active. ASK, recording and remaining modes SHALL follow in their existing order. At most three badges SHALL appear, separate from fixed playback slots. Color alone SHALL NOT identify a mode.

#### Scenario: Mode badges follow the active screen state
- **WHEN** keyboard or raw MIDI actions enter and leave existing modes on Pads, Files, Chop, FX and Settings
- **THEN** their title badges follow the actual current state on the next frame
- **AND** leaving a mode removes its badge without moving shared screen rows

#### Scenario: Badges retain identity without color
- **WHEN** title badges appear in a terminal with limited colors
- **THEN** their text still distinguishes mode, editing, learning, questions and recording

### Requirement: Existing mode badge labels

Placement labels SHALL be `IN HAND`, `COPY`, `SWAP` or `ASSIGN`. Marking SHALL show `MARK` or `N MARKED`, editing `EDIT` with its page, learning `LEARN` and questions `ASK`. Recording SHALL show `REC` with elapsed time when space permits. Chop toggles SHALL show `LOOP` and `AS PAD`. Mark and assignment badges SHALL retain counts. Inactive states SHALL NOT produce these mode labels. Transport feedback SHALL remain distinct from pad playback. Controller panels remain out of scope.

#### Scenario: Every existing mode has its badge
- **WHEN** each existing placement, marking, edit, learning, question, recording or Chop toggle state is active in a separate fixture
- **THEN** its badge uses the specified label, page, count or elapsed time
- **AND** transport feedback remains separate from pad-playback state without advertising a controller panel

#### Scenario: Overlapping placements remain visible
- **WHEN** more than one of file placement, copy and swap is pending
- **THEN** one compound PadMode badge names the pending placement modes
- **AND** existing keyboard and MIDI action priorities remain unchanged

### Requirement: Badge width and compatibility

At supported widths, badges SHALL retain screen identity, transport feedback, questions and the active PadMode label. Compact labels SHALL precede removal of lower-priority badges. File details SHALL yield space first. Crowded Chop headers MAY use `#i/n` for variants and `Nsl` for slice counts without losing these numbers. PLAY and LISTEN SHALL retain their ten-cell slots, styles, actual-playback semantics and stable following columns. Badges SHALL NOT change sound, keys, data or input order.

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

### Requirement: Accepted pattern clock feedback

Every screen SHALL identify accepted A1 playback and its loop-relative bar and beat beside the existing transport feedback. It SHALL use one coherent current-project audio snapshot for clock position, pattern presence and length. Transport position SHALL remain distinct when it exceeds the loop length. Pending replacement SHALL NOT show a false accepted length. Without a pattern, no A1 label SHALL appear.

#### Scenario: Pattern title follows the accepted loop across screens
- **WHEN** A1 repeats while the owner switches screens, tempo changes or a replacement waits for acceptance
- **THEN** the title follows accepted loop and transport positions without estimating time in the UI
- **AND** snapshots from an old project cannot show its pattern or loop length

### Requirement: Pattern feedback preserves layout and safety

Pattern feedback SHALL extend the existing transport segment, not create a fourth badge or move rows. At 80 by 26 and 120 by 40, screen identity, clock, questions, active pad mode and save or audio warnings SHALL keep their existing priority. Compact pattern identity SHALL use `A1`. The offline catalogue SHALL cover active, stopped, empty and pending pattern states.

#### Scenario: Pattern titles retain safety and actual playback slots
- **WHEN** pattern feedback competes with long names, ASK, EDIT, placement, recording or save and audio warnings at either supported size
- **THEN** the existing priority rules keep required feedback readable inside its bounds
- **AND** PLAY and LISTEN retain their ten-cell slots and shared rows stay fixed

### Requirement: Visible Pitch Gate state

The selected pad's mode row SHALL distinguish Off, On for A1 only, and On but inapplicable. When On, source or pitch restrictions SHALL have a text reason, not color alone. The Pads help row SHALL explain Pitch Gate only for a linked slice with Pitch Gate On. Scope, popup, edit pages and controls SHALL remain. Feedback SHALL NOT claim a cut from the saved switch, running transport or a manual hit. No top badge, row shift or new pad mode SHALL appear.

#### Scenario: Pitch Gate feedback preserves parameter pages
- **WHEN** keyboard or raw MIDI toggles Pitch Gate with edit open or closed, hidden or shown pads and eligible or ineligible sources
- **THEN** the panel, popup and F1 show the saved state, A1-only scope, applicable controls and any source or pitch restriction
- **AND** every existing page, knob target, safety indicator and input route stays available

#### Scenario: Pitch Gate help row appears only where it applies
- **WHEN** the selected pad is a whole file or a linked slice with Pitch Gate Off, or a linked slice with Pitch Gate On at zero or positive pitch
- **THEN** the help row explains Pitch Gate only for the linked slice with Pitch Gate On
- **AND** the panel shows the Pitch Gate label and F1 gives the reason for every pad

#### Scenario: Pitch Gate reports source and pitch restrictions
- **WHEN** Pitch Gate is On for an empty pad, a whole file, a detached piece or a linked slice at zero or negative pitch
- **THEN** feedback keeps the saved On value and identifies the source or pitch restriction in text
- **AND** a source or pitch change refreshes the explanation without automatically changing the saved switch

#### Scenario: Pitch Gate scope stays explicit during manual play
- **WHEN** an eligible pad has Pitch Gate On without A1, with stopped transport or during mixed manual and A1 playback
- **THEN** the indicator identifies the A1-only setting and the visible help states that manual hits stay unchanged
- **AND** a manual hit or running transport cannot turn that setting into a claim that the current sound has a cut

#### Scenario: Pitch Gate explanations fit the existing frame
- **WHEN** Pitch Gate states appear at 80×26 and 120×40 with hidden or shown pads, edit open or closed and safety warnings
- **THEN** the mode, compact Pitch Gate state and applicable text explanation remain readable in the existing panel and help rows
- **AND** the view adds no top badge and preserves the fixed rows, safety priority and access to existing controls

### Requirement: Sequenced pad feedback uses actual voices

Current-project sequenced voices SHALL count toward PLAY and pad hit feedback under existing actual-voice rules. Silent, dropped, queued and old-project events SHALL NOT count. Sequenced events SHALL NOT change selection, marks, held-input state, bank or keyboard/controller identity. Preview and effect tails alone SHALL retain their existing exclusions.

#### Scenario: Pattern voices light PLAY without pretending to be owner input
- **WHEN** real callbacks play and end A1 notes with grid hidden or shown and edit open or closed
- **THEN** PLAY follows current pad voices and applicable pad flashes follow actual starts
- **AND** selection, marks, bank, held inputs and KB/controller identity stay unchanged

### Requirement: Accepted pattern recording indicators

Every current screen SHALL show accepted count-in, `REC A1`, erasure or Finishing state without confusing it with microphone or output capture. Pending requests SHALL NOT claim an accepted phase. Phase, loop position and target SHALL come from one coherent current-project audio snapshot. Stale feedback SHALL NOT restore another project's recording label.

#### Scenario: Recording indicators follow accepted audio state
- **WHEN** a recording command waits, count-in advances, recording starts, erasure toggles and the pass finishes
- **THEN** each screen shows the matching accepted phase and A1 target without premature REC feedback
- **AND** stale or mixed-project feedback cannot change those shown indicators

### Requirement: Count in and loop position are visible

Count-in SHALL show `COUNT` and descending digits 4, 3, 2, 1 with each accepted beat. A1 recording SHALL show current bar, beat and loop length. Quantize grid and strength SHALL remain available without opening Settings. Feedback SHALL advance without owner input. The full step grid and editable pad rows SHALL NOT appear in 97.

#### Scenario: Count in becomes recording at tick zero
- **WHEN** stopped recording proceeds through four beats at either supported terminal size
- **THEN** visible count digits follow the audio beats and change to `REC A1` at the accepted pattern start
- **AND** position, length, grid and strength remain readable without a new step screen

### Requirement: Pattern work keeps the shared frame

Pattern feedback SHALL keep the fixed title, status, strip, grid and three-row help positions. Compact pattern controls SHALL show Bars, Quantize and Strength with a selected row and applicable edit restrictions. Count digits SHALL use available content space without covering questions or text input. Safety indicators SHALL retain their existing width priority.

#### Scenario: Pattern controls fit without moving the pads
- **WHEN** pattern controls open and close on each current screen at 80×26 and 120×40
- **THEN** the selected setting, current values and close controls remain readable
- **AND** screen identity, pad rows, save failures and audio warnings retain their fixed frame

### Requirement: Erasure has a distinct visible pad role

Erasure SHALL show `ERASE` with the A1 recording state. Its held targets SHALL have a distinct erase marker, not PLAY or ordinary held-note styling. The view SHALL identify the target bank and pad even with a hidden grid. Leaving erasure SHALL clear its target markers without a sounding-pad flash. No-release fallback SHALL name its one-loop target duration.

#### Scenario: Erasure remains distinct from sounding pads
- **WHEN** erasure targets pads across a bank change with the grid shown or hidden
- **THEN** ERASE and actual target identities remain visible without implying a manual sounding hit
- **AND** leaving erasure clears those marks and shows the return to ordinary recording

### Requirement: Recording catalogue covers supported states

The offline screen catalogue SHALL cover CountIn, Recording, Erase, Finishing and PatternControls on every applicable registered screen at both supported sizes. It SHALL include busy, blocked and safety-warning combinations. Missing entries SHALL fail the existing coverage check. Generation SHALL NOT open devices or edit projects.

#### Scenario: Catalogue includes recording states and safety combinations
- **WHEN** the offline catalogue generates after 97 implementation
- **THEN** both supported sizes contain every applicable recording phase and controls state
- **AND** the matrix detects missing entries and keeps save or audio warnings visible in crowded fixtures

### Requirement: Pads pattern event strip

F2 SHALL show three A1 event rows below its live output wave, within the shared strip rectangle. The rows SHALL be a header, an all-pads row and the selected pad's row. A strip of four or more rows SHALL keep at least one wave row. A three-row strip SHALL show only the event rows. Title, bank, pad grid, right column, help and safety status SHALL keep their positions. Other screens SHALL keep their strip contents. No A1 SHALL show a recording instruction without creating a pattern.

#### Scenario: Pattern strip fits the shared frame
- **WHEN** F2 shows absent, empty or populated A1 at 80x26 and 120x40, with pads shown or hidden
- **THEN** the event rows fit below the wave or replace it in a three-row strip, and the outer frame and warnings stay
- **AND** Chop and Files draw no event rows, and hidden pads give the strip more wave rows

### Requirement: Pattern strip identifies time and event density

A word header SHALL name A1, the bar and beat, the loop length, the open bank, the view division as `view`, and the next-hit quantize and strength as `quantize`. The pad row label SHALL be `pad` with the pad key, without the file name. The all row SHALL mark cells with an onset of any pad of the bank. Empty, aligned, off-grid and multiple-event cells SHALL look different, with dim empty cells. Same-tick rows SHALL stay visibly multiple. Drawing SHALL NOT change raw rows or settings.

#### Scenario: Pattern strip distinguishes subdivisions and event density
- **WHEN** straight and triplet pages contain aligned hits, between-line hits, collisions, empty cells and hits of other pads in the bank
- **THEN** their forms, beat backgrounds, bar lines, header words and the all-pads row identify the corresponding data
- **AND** drawing changes no raw row, velocity, duration or setting

#### Scenario: Strip header names view and quantize apart
- **WHEN** the view is 1/16, the quantize is 1/4 at 100 %, and the owner selects pad Q of bank 4 with a long file name
- **THEN** the header shows `view 1/16 · quantize 1/4 100 % · bank 4`, and the pad row label is `pad Q`
- **AND** the file name does not show in the strip

### Requirement: Pattern strip groups beats symmetrically

Each beat SHALL start with one space. Each cell SHALL take its glyph and one space, so every glyph has a space on both sides. Alternate beats SHALL have a darker background. A vertical line SHALL separate bars, and no line SHALL separate beats. Beat groups SHALL follow the straight or triplet division. The playback plate SHALL cover a glyph and its two neighbor spaces.

#### Scenario: Pattern strip centers glyphs in beat groups
- **WHEN** straight and triplet pages show one or more bars with the playback plate on an onset and on an empty cell
- **THEN** every glyph has one space on each side, alternate beats have a darker background and only bar lines separate groups
- **AND** the plate covers exactly three columns centered on the current glyph

### Requirement: Bank row shows where A1 plays

The bank row SHALL underline the number of each bank that has a visible A1 onset. During playback, each closed bank's pad map SHALL light, in white, the pads with an A1 onset in the current 1/16 playback step. The open bank SHALL keep its current flashes. The row SHALL NOT change the open bank. A missing or stale audio position SHALL light no pattern pad.

#### Scenario: Bank row shows pattern banks and their hits
- **WHEN** A1 has onsets on banks 1 and 3, bank 1 is open, and transport stops, then plays
- **THEN** the row underlines numbers 1 and 3, and during playback bank 3 lights the pads with an onset in the current step
- **AND** the open bank does not change, and a missing or stale position lights no pattern pad

### Requirement: Pattern playback plate follows the audio position

The playback plate SHALL use coherent current-project audio position, never a UI timer. It SHALL be a centered three-column light plate in both event rows, without an arrow, frame or extra row. The page SHALL follow the plate. A missing or stale snapshot, or one from an old project, SHALL show no plate and no motion.

#### Scenario: Pattern playback plate follows the audio position
- **WHEN** playback crosses cells and pages, and then the project epoch changes or the loop length no longer matches
- **THEN** the plate marks the cell of the audio position and its page
- **AND** a stale or mismatched position shows no plate

### Requirement: Pattern strip yields to overlays and keeps safety visible

Pad-parameter curves and audio recording SHALL keep their use of the strip, and the event rows SHALL NOT draw then. The splash SHALL use only the wave rows above the event rows. A strip without wave rows SHALL show the event rows and no splash. Save, audio and xrun warnings SHALL stay visible with the event rows at both supported sizes. F2 SHALL keep terminal-cell drawing. Only Chop collects kitty images.

#### Scenario: Pattern strip yields to overlays and keeps safety visible
- **WHEN** F2 shows A1 with a pad-parameter curve, audio recording, or save and audio warnings
- **THEN** curves and the recording wave take the strip without event rows
- **AND** warnings stay visible next to the event rows at 80x26 and 120x40

#### Scenario: The splash leaves the event rows visible
- **WHEN** the program starts with and without A1 at 80x26 and 120x40
- **THEN** the event rows or the recording instruction show at both sizes
- **AND** the splash shows in the wave rows at 120x40, and a three-row strip at 80x26 shows no splash

### Requirement: Pattern view catalogue states

The offline catalogue SHALL cover the pattern view at both supported sizes. It SHALL include straight and triplet views, off-grid and multiple events, the hidden-grid preference and safety warnings. Missing required entries SHALL fail coverage. Generation SHALL NOT open audio or MIDI devices or edit projects.

#### Scenario: Catalogue covers the pattern view
- **WHEN** the offline catalogue generates
- **THEN** both sizes contain the required pattern-view entries and a missing entry fails coverage
- **AND** generation leaves projects and device access untouched

### Requirement: Strip header explains multiple hits

When a visible cell of the selected pad holds two or more hits, the strip header SHALL end with `2-9 = hits in a cell, one plays`. Without such a cell, the header SHALL NOT show it. When the width is too small, the legend SHALL be cut before the bank.

#### Scenario: Strip header explains multiple hits
- **WHEN** the visible page of the selected pad has a cell with two hits, and then has none
- **THEN** the header ends with the legend only while such a cell is visible
- **AND** the bank stays visible at 80 columns

### Requirement: Whole-pad erasure shows hit counts

In whole-pad erasure, the Pads title SHALL show `ERASE`. Each pad of the open bank with A1 rows SHALL show `N hits` on its mode line. N SHALL count every raw row that a press erases, hidden rows included. A pad without rows SHALL show an empty mode line. After an erasure, the count SHALL go away at the next redraw. Leaving erasure SHALL restore the usual mode lines. The counts SHALL fit 80x26 and 120x40.

#### Scenario: Erasure shows what a press erases
- **WHEN** whole-pad erasure is on with 4 hits on pad Q, 2 hits on pad W and none on pad E, at 80x26 and 120x40
- **THEN** the title shows `ERASE`, pad Q shows `4 hits`, pad W shows `2 hits`, and pad E shows no count
- **AND** after a press on pad Q its count goes away, and leaving erasure restores the mode lines

### Requirement: Pattern controls take the side panel

While pattern controls are open, the right column SHALL show only the A1 controls in the style of the pad panel. Block lines SHALL name `A1 PATTERN` and `CLEAR`. Labels SHALL be lowercase, and the selected row SHALL be bright. The keys SHALL show dim at the bottom of the column. The pad panel SHALL NOT show under the controls. Closing SHALL restore the screen's own column. Title, bank, pad grid, strip and help SHALL keep their positions.

#### Scenario: Pattern controls replace the pad panel
- **WHEN** the owner opens pattern controls on Pads, Chop, Files and FX at 80x26 and 120x40
- **THEN** the right column shows the A1 PATTERN and CLEAR blocks and the keys, and no pad parameter
- **AND** closing restores the pad panel and every other position stays

### Requirement: Pattern controls use the pad edit frame

Pattern controls SHALL use the frame of the pad edit: the edit color and the title `EDIT ·` with the block names `PATTERN` and `CLEAR`. The block of the selected row SHALL be lit, as the open page of the pad edit is. A narrow column SHALL drop `EDIT ·` before the block names, as the pad edit does.

#### Scenario: Pattern controls use the pad edit frame
- **WHEN** the owner opens pattern controls and selects Bars, then Clear A1, at 80x26 and 120x40
- **THEN** the column has the edit frame with `PATTERN` lit, then with `CLEAR` lit
- **AND** the frame has the color of the pad edit frame

### Requirement: Pattern controls show restrictions in place

The edit restriction of the selected row SHALL show dim next to its value. A refused edit SHALL show the restriction in the warning color.

#### Scenario: Pattern controls show restrictions in place
- **WHEN** the owner selects Strength, then tries to change Bars during Recording
- **THEN** `step 10` shows dim next to Strength, and the Bars refusal shows in the warning color

### Requirement: The Chop slice line keeps each value whole

The Chop slice line SHALL wrap only between its ` · ` parts. A number SHALL stay on one row with its unit. A part wider than the column SHALL wrap by words. While the slice edit is open, the edit frame SHALL take the place of the slice line. "The slice edit uses the pad edit frame" sets that frame.

#### Scenario: The Chop slice line keeps each value whole
- **WHEN** Chop shows the slice line at 80x26 and 120x40 with the slice edit closed
- **THEN** each of `start`, `length` and `cursor` stays on one row with its number and unit
- **AND** no text of the line goes away

### Requirement: Pattern labels and hints stay whole

The A1 strip SHALL show the pad row label `pad` with its key whole at 80x26 and 120x40. In pattern controls, a row hint that does not fit beside its value SHALL take its own line under the value. The key line SHALL name Enter as run and close.

#### Scenario: Pattern labels and hints stay whole
- **WHEN** the owner selects pad Q with A1 hits, opens pattern controls and selects quantize, strength and clear A1 at 80x26 and 120x40
- **THEN** the strip shows `pad Q`, and `next hits`, `next hits, step 10` and `Enter` show whole
- **AND** the key line says `Enter run`
