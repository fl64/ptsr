# projects Specification

## Purpose
Keep each project in its own folder with saved pads, chops and sample paths. Support project selection, persistent root folders and isolated sound when projects change.

## Requirements

### Requirement: Project is a folder
The system SHALL keep a project as one folder: `project.toml` holds the session of that project (pad samples, modes, volumes of all banks, chops, the browser folder), `samples/` holds material born in the program — a recording goes there while the project is open, to the samples root (settings) without a project; sample paths inside the folder are stored relative to the project folder, paths outside stay absolute; no file is copied into the project behind the owner's back.

#### Scenario: A recording lands in the project
- **WHEN** the project "beat1" is open and the owner stops a recording
- **THEN** the WAV is in `beat1/samples/`, and the pad that gets it stores the path relative to the project; without a project the WAV is in the samples root set in the settings

#### Scenario: A new project folder
- **WHEN** the owner creates the project "MyBeat" in the picker
- **THEN** the projects root contains the folder `MyBeat` with `project.toml` and an empty `samples/`

#### Scenario: Relative paths inside the folder
- **WHEN** a pad plays a sample that lies inside the open project's folder and the session saves
- **THEN** `project.toml` stores that sample's path relative to the project folder

#### Scenario: Moving the project
- **WHEN** the whole project folder is moved and the owner opens the project from the new place
- **THEN** every pad sounds the same as before the move

#### Scenario: Outside stays absolute and untouched
- **WHEN** a pad plays a sample that lies outside the project folder
- **THEN** `project.toml` stores its absolute path and the file is not copied into the project

### Requirement: Project picker
The system SHALL open a project picker on `Ctrl+P` listing the folders of the projects root that contain a `project.toml`; `Enter` opens the picked project; typing a name opens the name input line and on `Enter` creates a new project and opens it; while a project is open its name shows in the window title and in the status line of every screen.

#### Scenario: Open a project
- **WHEN** the owner presses `Ctrl+P` and `Enter` on "MyBeat"
- **THEN** the session of MyBeat loads, its pads and chops are on the screen, and the title and the status line show "MyBeat"

#### Scenario: Create a project from the picker
- **WHEN** the owner types "NewBeat" in the picker and presses `Enter`
- **THEN** the folder with `project.toml` and `samples/` appears in the projects root and the project opens

#### Scenario: Switch and switch back
- **WHEN** the owner switches from one project to another and back
- **THEN** each project shows exactly what it saved: its pads, banks, chops and browser folder, nothing leaks between projects

#### Scenario: Folders without a project
- **WHEN** the projects root contains folders without a `project.toml`
- **THEN** the picker does not list them

### Requirement: The last project reopens
The system SHALL remember the last opened project and open it on the next start; without any project ever opened the program works in the scratch session exactly as today and nothing asks for a project.

#### Scenario: Restart with a project
- **WHEN** the owner closes the program with a project open and starts it again
- **THEN** that project opens with all its pads and chops as saved

#### Scenario: First start is scratch
- **WHEN** the program starts and no project has ever been opened
- **THEN** the scratch session works and is saved as today, with no project dialog in the way

### Requirement: Deleted project folder

The system SHALL keep working when the open project's folder disappears from disk.
It SHALL mark pads with files inside that folder as "missing file" and drop the project name from the window title.
It SHALL refuse saving without recreating the project folder.
The refusal SHALL retain pending work and allow a manual retry if the folder becomes available again.

#### Scenario: Folder deleted while open
- **WHEN** the folder of the open project is deleted while the program runs
- **THEN** the program keeps working, the pads are marked as "missing file", and the title shows no project name

#### Scenario: Saving does not recreate a deleted project
- **WHEN** the project folder disappears before or during a save
- **THEN** saving fails without recreating the folder and pending work remains in memory
- **AND** each manual save request reports the refusal

### Requirement: Roots in the settings
The system SHALL offer two rows in the settings screen (F12): the projects root (default `~/ptsr/projects`) and the samples root (default `~/ptsr/samples`); each row opens a directory picker that walks the folders and picks with `Enter`; the choices survive a restart.

#### Scenario: Pick the roots
- **WHEN** the owner opens a root row, walks the picker into a folder and presses `Enter`
- **THEN** the row shows the picked folder and the picker closes

#### Scenario: The choice survives a restart
- **WHEN** the owner changes a root and restarts the program
- **THEN** the row shows the picked folder again

### Requirement: The open project is marked
The system SHALL mark in the project picker the project that is open now, and SHALL show under the list the keys of the picker, not the keys of the screen beneath it.

#### Scenario: Which one is open
- **WHEN** "beat1" is open and the owner presses `Ctrl+P`
- **THEN** the row of "beat1" says it is open, and the help line names the picker keys: select, open, type a name to create, close

### Requirement: Current program files

The system SHALL use `ptsr` for its executable, window title, MIDI client and sound client (`ptsr:out_l`, `ptsr:out_r`, `ptsr:in_l`, `ptsr:in_r`).
Folders SHALL be `~/.config/ptsr/` for config, keymap and MIDI bindings, `~/.local/share/ptsr/` for session, and `~/.local/state/ptsr/` for log.
`XDG_CONFIG_HOME`, `XDG_DATA_HOME` and `XDG_STATE_HOME` SHALL relocate them as before.

#### Scenario: Current ptsr files load
- **WHEN** current `ptsr` settings, keys and session exist alongside conflicting `arasaka-404` files
- **THEN** the current files load without importing or changing the old files
- **AND** current sound-device settings, MIDI bindings, owner keys and session pad state remain available

### Requirement: Legacy program folders are not imported

Startup SHALL NOT inspect, copy, rename or remove `arasaka-404` config or session folders, or show a settings-moved notice.
Absent current files SHALL use the existing default settings, shipped keys and empty scratch session.
Current files and XDG paths SHALL keep their existing behavior.

#### Scenario: Legacy folders stay unused without current files
- **WHEN** only legacy `arasaka-404` settings, keys and session files exist
- **THEN** current-file loaders use defaults, shipped keys and an empty session without creating current folders by copying legacy files
- **AND** legacy files remain unchanged

### Requirement: Project sounds stay separate

The system SHALL play only the sounds of the open project on new pad hits. An empty or
missing pad SHALL be silent. Loading from an earlier project SHALL NOT change the open
project, even if it finishes after the switch. Each pad SHALL use the settings of the
open project. These rules apply to keyboard and MIDI hits. On a project switch, sounds
already playing from the old project SHALL fade out without a click. Until the new project
is ready, pad hits SHALL be silent and SHALL NOT play later.

#### Scenario: Empty pads after a project switch
- **WHEN** the owner switches from a project with samples and slices to a new empty project
  and hits the same pads
- **THEN** the pads are silent, even when the audio command queue was full during the switch

#### Scenario: A late sample from the old project
- **WHEN** a sample load from the old project finishes after a switch to an empty project
- **THEN** the empty pad stays empty and silent

#### Scenario: Playing sounds stop at a project switch
- **WHEN** a pad sounds and the owner opens a different project
- **THEN** the old sound fades out without a click and stays silent

#### Scenario: Hits while the project opens
- **WHEN** the owner hits pads while the new project is not ready
- **THEN** the hits are silent and do not play later

#### Scenario: Return to a project with sounds
- **WHEN** the owner switches from a project with samples and slices to an empty project
  and then returns to the first project
- **THEN** its pads play their own samples and slices with their saved settings

### Requirement: Read protection survives manual save requests

A session file that fails to load SHALL remain protected against saving.
This protection SHALL apply to damaged files, unreadable files and files from a newer format version.
Manual save requests SHALL report the protection reason without replacing the file.
A write failure alone SHALL NOT establish read protection.

#### Scenario: Damaged or newer session remains protected
- **WHEN** startup cannot load a damaged or newer project or scratch session and the owner requests manual saves
- **THEN** each request explains the refusal and the protected file remains byte for byte unchanged
- **AND** edits do not clear the protection or pending work

### Requirement: Save before project transitions

Opening another project or creating a new project SHALL save pending source work before replacing the current session.
Source work SHALL include scratch and named projects.
A save refusal SHALL keep pads, chops, undo, pending work, picker selection and the typed name.
New target folders and files SHALL NOT be created before a required source save succeeds.
Existing keys, startup read protection and sound isolation SHALL stay unchanged.

#### Scenario: Project edits survive an immediate switch
- **WHEN** the owner changes a project and selects another project before autosave
- **THEN** the source saves before the target replaces it
- **AND** returning to the source restores the saved pad and chop changes

#### Scenario: A save refusal keeps the selected target
- **WHEN** the owner selects another project while pending source work cannot save
- **THEN** each Enter reports refusal and retains the current pads, chops, undo and pending timestamp
- **AND** the picker stays open with the same selection and the target stays unchanged

#### Scenario: A save refusal keeps the new project name
- **WHEN** the owner enters a new project name while pending source work cannot save
- **THEN** each Enter retains the typed name and current work without creating that target
- **AND** after the write problem ends, Enter saves the source and creates the target

### Requirement: Selecting the current project does not reload

Selecting the current project's folder SHALL close the picker without reading, saving or reloading the session.
Pending work, undo, loader generations and current sound state SHALL stay unchanged.
Equivalent canonical folder paths SHALL count as the same folder when both paths exist.

#### Scenario: Selecting the current project keeps pending work
- **WHEN** the owner selects the open project, including a folder alias, while changes are pending
- **THEN** the picker closes without losing the changes or undo history
- **AND** no session save, loader reset or audio epoch change occurs

### Requirement: Validate a target before replacing the current project

An existing target SHALL pass session reading and validation before source saving or state replacement.
A damaged, newer, unreadable or missing target session SHALL keep the source project, save flags and last-project setting unchanged.
A target-creation refusal after source saving SHALL keep the source and picker open, with that source work safely saved.

#### Scenario: A target read refusal preserves the source
- **WHEN** the owner selects a target whose session is damaged, newer, unreadable or missing
- **THEN** the current session, pending work, undo, picker selection and last-project setting stay unchanged
- **AND** the refusal does not write either session or replace the source read-protection reason

#### Scenario: A target creation refusal retains saved source work
- **WHEN** source saving succeeds but the new target cannot be created
- **THEN** the source and its undo remain available and its pending flag reflects the successful save
- **AND** the picker keeps the typed name for a later retry

### Requirement: Project paths include every saved chop

Project path conversion SHALL include pads, the current chops and every saved older chop variant.
Paths inside the project SHALL store relative to its folder and resolve against its current folder on load.
External absolute paths SHALL stay unchanged. No sample SHALL be copied.
The saved format and scratch path behavior SHALL stay unchanged.

#### Scenario: Internal variant paths move with the project
- **WHEN** a project with several saved variants of an internal sample is saved, moved as a whole folder and opened from its new location
- **THEN** pads, newest chops and every older variant use the sample in the new folder
- **AND** saving stores their internal paths relative to that folder

#### Scenario: External variant paths remain absolute
- **WHEN** a project with several variants of an external sample is saved, moved and reopened
- **THEN** all saved variant keys and pad paths keep the same external absolute path
- **AND** the external file stays byte for byte unchanged and no sample copy appears in the project

#### Scenario: Save and reopen after project relocation
- **WHEN** the moved project is saved and opened again
- **THEN** all saved variants remain available with their full state and correct sample paths

### Requirement: Portable chop compatibility

Projects without saved older variants SHALL keep the existing pad-bound fallback.
Stale absolute paths from an earlier move SHALL NOT be replaced by guessed paths.
This change SHALL NOT migrate the saved format or search for replacement samples.

#### Scenario: Missing kept variants use the existing fallback
- **WHEN** an old project has a linked pad but no saved variant for its epoch and the owner opens that pad with F3
- **THEN** the existing fallback rebuilds the chop from its pad bounds without a format migration

#### Scenario: Stale absolute variant paths are not guessed
- **WHEN** a project already contains an older variant key that names an absent old absolute location
- **THEN** loading leaves that key unchanged instead of mapping it to a same-named sample in the new folder
- **AND** the existing fallback remains available for pads whose saved variant cannot be found

### Requirement: Saved transport preferences

Project and scratch session format 5 SHALL save `bpm` and `metronome`. Tempo SHALL be finite and within 40–300 BPM, rounded to 0.1 BPM. New sessions SHALL use 120 BPM and metronome off. Playback position and running state SHALL NOT be saved. Tempo and metronome edits SHALL use existing dirty-state, manual-save and retry rules.

#### Scenario: Project transport preferences survive a restart
- **WHEN** the owner saves 172.3 BPM with the metronome enabled and restarts
- **THEN** the project restores those preferences stopped at bar 1, beat 1

#### Scenario: Scratch transport preferences survive a restart
- **WHEN** the owner changes tempo and metronome without an open project and saves
- **THEN** reopening scratch restores its preferences without starting playback

### Requirement: Transport session migration

Formats 1–4 SHALL remain readable and use 120 BPM and metronome off when transport fields are absent. Existing pad, chop, group, name, path and effect state SHALL remain intact through migration and saving as format 5. Newer formats and invalid transport values SHALL retain existing load refusal and read protection.

#### Scenario: Older sessions gain transport defaults without changing pads
- **WHEN** supported format 1, 2 or 3 sessions load and then save
- **THEN** format 5 contains transport defaults and preserves the existing migrated pad and chop state
- **AND** groups, effects, copy names and stored paths keep their existing meanings

#### Scenario: Invalid transport settings protect the session file
- **WHEN** a target session contains non-finite tempo, tempo outside the supported range or an invalid metronome value
- **THEN** loading refuses it without replacing current work
- **AND** startup load refusal protects the original file from later save requests

### Requirement: Transport isolation during project changes

A validated project switch SHALL stop and reset the source clock, discard its queued transport commands and restore the target's preferences before new transport starts. A refused read or required source save SHALL leave the current clock and preferences unchanged. Selecting the current project SHALL NOT restart it.

#### Scenario: Different projects retain independent transport settings
- **WHEN** the owner switches between projects with different transport preferences while the audio queue is full
- **THEN** the target opens stopped with its own preferences and no old command changes them later

#### Scenario: Refused project switch preserves running transport
- **WHEN** target validation or a required source save refuses a project switch while transport runs
- **THEN** the source keeps its tempo, metronome preference and running position without a reset

#### Scenario: Selecting the current project preserves its running clock
- **WHEN** the owner selects the already open project or its canonical folder alias
- **THEN** the picker closes without stopping, resetting or restoring transport

### Requirement: Saved pattern and Pitch Gate state

Format 5 SHALL save zero patterns or one `A1` entry, its raw hits, bars, quantize, strength and selection. It SHALL save the per-pad `pitch_gate` boolean, with true as its absent default. Project and scratch saves SHALL use their existing safe replacement, dirty-state, retry and read-protection rules. Runtime notes, deadlines and running position SHALL NOT be saved.

#### Scenario: Pattern and Pitch Gate survive project and scratch restart
- **WHEN** project or scratch work saves A1 with raw off-grid and hidden hits, custom playback settings and mixed Pitch Gate values
- **THEN** reopening restores those exact data with transport stopped at tick zero
- **AND** the existing tempo, metronome, sounds, sources and mute groups retain their meanings

### Requirement: Pattern migration preserves older work

Formats 1–4 without patterns SHALL restore no pattern. A format 1–4 file with pattern selection or entries SHALL be refused without discarding those data. Format 4 SHALL retain its saved transport preferences. Earlier formats SHALL keep their existing migration rules. Absent Pitch Gate state SHALL default to true without changing manual playback. Saving as format 5 SHALL preserve names, groups, assignments, bounds, paths, chops, effects and ungrouped overlap.

#### Scenario: Format four gains pattern defaults without changing sound
- **WHEN** a format 4 project with tempo, metronome, linked slices and manual mute groups loads and saves as format 5
- **THEN** no pattern starts, transport preferences remain and absent Pitch Gate values become true
- **AND** existing manual renders match their pre-migration sound and ungrouped pads still overlap

#### Scenario: Older format numbers cannot hide pattern data
- **WHEN** a file declares format 1, 2, 3 or 4 but contains a pattern selection or entry
- **THEN** startup and project loading refuse it without changing the original bytes
- **AND** startup read protection and current project work remain intact

#### Scenario: Earlier session migrations remain compatible with patterns
- **WHEN** fixtures for formats 1, 2 and 3 load and save as format 5
- **THEN** their existing migrations preserve pad and chop behavior with no pattern and the specified transport defaults

### Requirement: Pattern file validation protects work

Loading SHALL reject invalid pattern ranges, malformed hit rows, duplicate or unsupported slots, mismatched selection and non-boolean Pitch Gate values. Invalid raw data SHALL NOT be silently clamped or discarded. A target refusal SHALL retain the current session and pending work. A startup refusal SHALL protect the file from later saves, including newer format versions.

#### Scenario: Invalid pattern targets do not replace a playing project
- **WHEN** a selected target contains invalid pattern settings, rows, selection, slots or Pitch Gate values
- **THEN** target validation refuses it before source saving or replacement
- **AND** the current rhythm, pending work, undo and picker selection remain unchanged

#### Scenario: Invalid or newer pattern startup files remain protected
- **WHEN** startup cannot read an invalid pattern session or a session newer than format 5
- **THEN** later saves explain the refusal and leave the original bytes unchanged

### Requirement: Patterns remain isolated across project transitions

A successful project transition SHALL clear source pattern playback and pending automatic releases before target readiness. The target SHALL open stopped with its own pattern data. Old queued patterns, swaps and releases SHALL NOT change it. A read or required source-save refusal SHALL leave the source rhythm running. Selecting the current folder SHALL NOT reload or reset it.

#### Scenario: Pattern project switches reject stale work under queue pressure
- **WHEN** projects with different patterns switch while command or retirement queues are full and old messages arrive late
- **THEN** only the target pattern can start after readiness and no source hit or deadline reaches it
- **AND** switching to an empty project produces no sequenced sound

#### Scenario: Refused or same-folder selection preserves the pattern
- **WHEN** a running project transition fails target reading or source saving, or selects its current canonical folder
- **THEN** its pattern and clock continue without a reset and current work remains available

### Requirement: Recorded patterns reuse format five

Completed pattern passes SHALL save through the existing format 5 raw-hit and settings fields. Runtime capture identity, count-in, erasure, pending gates and active recording SHALL NOT be stored. Load SHALL restore the raw take and settings with stopped transport and Idle recording. This change SHALL add no format migration or owner-project cleanup.

#### Scenario: Save and reopen a recorded rhythm without recording state
- **WHEN** a recorded and edited A1 saves and its project or scratch session reopens
- **THEN** raw rows, length, grid and strength match the completed take and transport remains stopped
- **AND** no recording phase or pending gate resumes and the saved format remains 5

### Requirement: Manual save completes the current pass

During pattern work, manual save SHALL first cancel CountIn or finish Recording and recover accepted capture through that boundary. It SHALL then invoke normal saving. It SHALL NOT report success before write success. Recording completion SHALL leave playback running. Count-in cancellation SHALL leave transport stopped. Refusal SHALL keep the completed take, Undo and pending save state.

#### Scenario: Manual save waits for a complete recording boundary
- **WHEN** Ctrl+S or a learned save button is pressed with recorded feedback and Gate completion pending
- **THEN** the pass finishes and saves its complete accepted rows before `Session saved` appears
- **AND** playback continues without resuming recording or writing runtime-only fields

A pending manual save SHALL keep its original input source and project ownership. Repeated save input SHALL join that request. A new pattern pass SHALL wait until the save attempt completes. Input processing, manual pads and releases SHALL continue while capture finishes. Autosave SHALL NOT replace the pending manual attempt. Project restoration SHALL discard an earlier project's request before it can write or report success.

#### Scenario: Repeated save waits for the same recording boundary
- **WHEN** the user requests another save while a manual save waits for delayed capture completion or a full command queue
- **THEN** only one save attempt and one pass Undo step result, with no early success notice
- **AND** manual pads and releases still work, a new recording pass waits, and the original request keeps its input source

#### Scenario: A delayed manual save cannot write another project
- **WHEN** project restoration replaces the source state before its pending manual save completes
- **THEN** the old request cannot write the new project or produce a save-success notice for it
- **AND** a later explicit save uses only the current project's state

The save request SHALL remove only its own progress notice when it completes or cancels. The progress notice SHALL NOT resume after the result notice expires. Removal of progress SHALL NOT change other notices or their existing display and history policies.

#### Scenario: Save completion removes hidden progress without hiding other notices
- **WHEN** another warning or error hides the progress notice before the pending manual save completes
- **THEN** completion removes only that request's progress notice, while other notices follow their existing display and history policies
- **AND** the success or refusal follows existing priority, while obsolete progress never resumes

#### Scenario: Save refusal retains a completed recording for retry
- **WHEN** a manual save ends a pass but read protection, a write failure or a missing folder refuses the write
- **THEN** the completed take, its Undo step and pending status remain and no false success appears
- **AND** an allowed retry saves the same recoverable work without recreating a missing project folder

### Requirement: Autosave and exit respect recording boundaries

Autosave SHALL wait during CountIn, Recording and Finishing. Quit SHALL complete or cancel that phase before applying existing exit-save policy. Audio loss SHALL permit recovery without waiting for a callback that cannot arrive. A write refusal SHALL retain existing warning and retry semantics. This change SHALL NOT promise recovery of unsaved data after a process crash.

#### Scenario: Autosave cannot write a partial gate take
- **WHEN** autosave time passes during count-in or recording with unfinished Gate notes
- **THEN** the last saved file stays unchanged and pending save status remains visible
- **AND** after the pass finishes ordinary autosave can save complete rows

### Requirement: Quit closes a recoverable pattern prefix

Quit SHALL freeze App capture at its last coherent accepted audio boundary. It SHALL complete recoverable rows without a new callback or acknowledgement. Later events SHALL NOT extend that prefix. Concurrent unavailable reads SHALL retry on normal UI turns without blocking input. Normal quit SHALL NOT set audio loss. A pending manual save SHALL keep its single attempt and source. Existing exit-save policy SHALL apply without an automatic retry after failure.

#### Scenario: Quit finalizes the recoverable pass before exit saving
- **WHEN** quit is requested during pattern work, with or without a final audio callback
- **THEN** recoverable capture is complete before existing exit-save policy runs
- **AND** no pending Gate duration or active recording phase enters the saved session

### Requirement: Project selection waits until pattern work ends

Project picker entry, direct open and creation SHALL refuse active phases, a requested start or pending quit before target validation, source saving or filesystem changes. Refusal SHALL preserve the take and clock and explain how to finish first. After completion, existing Idle transition rules SHALL apply. New-project state SHALL NOT accept capture, releases or controls from an earlier project or pass.

#### Scenario: A project request cannot interrupt an active take
- **WHEN** keyboard or raw MIDI requests project selection during any active recording phase
- **THEN** the current take, screen and clock remain and the notice explains how to finish recording
- **AND** after completion the picker uses existing transition safety without carrying recording state into another project
