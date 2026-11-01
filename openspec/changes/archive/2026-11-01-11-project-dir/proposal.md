# 11 — Projects: a folder with its samples

## Why

Samples are scattered over the disk and the session keeps absolute paths: the set cannot move, and recordings have no home. A project is a folder that keeps the bindings and the material together.

## What Changes

- **A project is a folder**: `project.toml` (pad bindings, volumes, modes, chops — the session, with paths relative to the folder) and `samples/` (what was born in the program: recordings, saved edits).
- **Two configurable roots** in the settings, each with a directory picker dialog: the projects root (default `~/ptsr/projects`) and the samples root (default `~/ptsr/samples`). The files screen starts in the samples root. The user config itself lives at `~/ptsr/config.toml` (moving it there is the rename change's job).
- **Ctrl+P** opens the project picker: the folders of the projects root that hold a `project.toml`; Enter opens it, a command creates a new project. The project name shows in the window title.
- **No project — no change**: the session without a project stays as it is today (a scratch pad).
- **Bindings use relative paths** for samples inside the project folder, absolute for the rest; nothing is copied behind the owner's back.
- Recordings and saved edits go to the open project's `samples/`, or to the samples root without a project.

## Capabilities

New: `projects`. Modified: `sample-loading` (the files screen starts in the samples root), `pad-assignment` (the session is saved into the open project).

## Out of scope

- Recording (a separate change); renaming the program (a separate change).
- Migrating the existing scratch session into a project (the owner does it by creating one and putting samples again, or we add it later on request).

## How to verify

1. F12: two new rows, Projects root and Samples root; the picker walks the folders, Enter picks; the choice survives a restart.
2. Ctrl+P: the picker lists the projects root; create "MyBeat": the folder `~/ptsr/projects/MyBeat` appears with `project.toml` and `samples/`; the title shows "MyBeat".
3. Put samples on pads, chop one, restart with the project open: everything is back; `project.toml` has relative paths for the samples inside the folder.
4. Ctrl+P to another project and back: the pads switch accordingly, nothing leaks between projects.
5. Delete a project folder while it is open: the program keeps working, the pads say "missing file", the title drops the name.
