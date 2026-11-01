# Design 11

## Model

- A project is a folder; `project.toml` is the same session document the scratch session
  uses today (same format, same version line). The only difference is the destination: with
  a project open, the existing save path writes into the project folder, the scratch file
  stays untouched.
- Paths: one helper, applied at save time — a path inside the project folder is stored
  relative to the folder, a path outside stays absolute. On load the mirror rule resolves
  relative paths against the project folder. The rule covers pad sample paths and the
  browser folder.
- The open project is app state: name + folder path (`Option<...>`, `None` = scratch).
- The last opened project name persists in the config as a `[project]` section (the
  existing `save_section` mechanism appends it without touching other sections).
- Roots: two config fields, defaults `~/ptsr/projects` and `~/ptsr/samples`; the root
  folders are created empty on first use if missing. The F12 rows reuse the settings
  screen; the directory picker is a small dialog with the files-screen navigation model
  (→ into, ← up, Enter picks, Esc cancels).
- The picker (`Ctrl+P`) lists the projects-root folders holding a `project.toml`. The
  create command opens a name input line (the filter-line pattern); Enter creates the
  folder with `project.toml` + `samples/` and opens it.
- Opening a project (start, picker, switch) goes through the same restore path as today's
  start restore: the old session is dropped, the new one's samples load in the background.
  Nothing of the old project stays in memory — that is the "no leaks" guarantee.
- Deleted folder while open: detected when a save is attempted; the save refuses to run
  (a plain write would re-create the folder, since saving creates missing parent
  folders), the session stays in memory, the pads are marked "missing file", the title
  drops the name, the program keeps running.

## Decisions (with the owner, 2026-10-31)

- Start reopens the last project automatically; the config keeps its name.
- No way back to scratch after the first project; the scratch session lives only until
  then.
- A new project is created by a name input line in the picker, Enter creates and opens.

## Open questions

None. Defaults taken: roots are created empty on first use; the scratch session keeps
today's path and format; migrating the scratch session into a project stays out of scope
(proposal).
