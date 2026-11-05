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
