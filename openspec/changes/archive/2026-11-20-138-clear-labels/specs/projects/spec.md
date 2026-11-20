## ADDED Requirements

### Requirement: A new project from the controller

When the projects root has no project, the project picker SHALL show `new project` on device pad P4. A press SHALL make and open a project named by the date and the first free number, for example `2026-11-20-1`. The owner can rename it later from the keyboard. With projects in the list, P4 SHALL open the selected project, as before.

#### Scenario: New project with no projects
- **WHEN** the projects root has no project on 2026-11-20 and the owner presses PAD BANK with device pad P4 in the project picker
- **THEN** the project `2026-11-20-1` is made and opens

#### Scenario: The date name takes the next number
- **WHEN** the projects root has no project but has a folder `2026-11-20-1`, and the owner makes a new project from the controller on that date
- **THEN** the new project is `2026-11-20-2`

## MODIFIED Requirements

### Requirement: Project is a folder
The system SHALL keep a project as one folder: `project.toml` holds the session of that project (pad samples, modes, volumes and sound gains of all banks, chops, the browser folder), `samples/` holds material born in the program — a recording goes there while the project is open, to the samples root (settings) without a project; sample paths inside the folder are stored relative to the project folder, paths outside stay absolute; no file is copied into the project behind the owner's back. A session without a sound gain SHALL load with 0 dB.

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
