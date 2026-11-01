# projects Specification (delta)

## ADDED Requirements

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
The system SHALL keep working when the open project's folder disappears from disk: the pads are marked as "missing file" and the title drops the project name.

#### Scenario: Folder deleted while open
- **WHEN** the folder of the open project is deleted while the program runs
- **THEN** the program keeps working, the pads are marked as "missing file", and the title shows no project name

### Requirement: Roots in the settings
The system SHALL offer two rows in the settings screen (F12): the projects root (default `~/ptsr/projects`) and the samples root (default `~/ptsr/samples`); each row opens a directory picker that walks the folders and picks with `Enter`; the choices survive a restart.

#### Scenario: Pick the roots
- **WHEN** the owner opens a root row, walks the picker into a folder and presses `Enter`
- **THEN** the row shows the picked folder and the picker closes

#### Scenario: The choice survives a restart
- **WHEN** the owner changes a root and restarts the program
- **THEN** the row shows the picked folder again
