# Tasks — 11

- [x] 1. Roots in the settings: the two F12 rows with the directory picker dialog, the
  `[project]`/roots config fields with defaults `~/ptsr/projects`, `~/ptsr/samples`, the
  roots created empty on first use; tests named after the scenarios `Pick the roots`,
  `The choice survives a restart`; commit.
- [x] 2. The relative-path helper: inside the folder — relative, outside — absolute, with
  the mirror resolution on load; round-trip tests `Relative paths inside the folder`,
  `Outside stays absolute and untouched`, `Moving the project`; commit.
- [x] 3. The picker and the project state: `Ctrl+P` lists the root folders holding a
  `project.toml`, the create-command name line creates and opens, the title shows the
  name, the last project reopens on start, the scratch session stays untouched until the
  first project; recorded-event tests `Open a project`, `Create a project from the
  picker`, `Switch and switch back`, `Folders without a project`, `Restart with a
  project`, `First start is scratch`, `A new project folder`; commit.
- [x] 4. The session destination: with a project open the autosave and `Ctrl+S` write the
  project's `project.toml` (relative paths), the scratch file untouched; the deleted
  folder while open keeps the program working with "missing file" pads and a dropped
  title; tests `Saved into the project`, `Restart`, `File is gone`,
  `Folder deleted while open`; commit.
- [x] 5. The files screen start: the samples root on first start, the project's `samples/`
  with a project open; tests `First start in the samples root`,
  `The project's samples come first`; commit.
- [x] 5a. A switch leaves nothing in the sound (owner, 2026-10-31: pad D played 7
  semitones down with +0 st on screen after a switch): the session restore sends every
  pad's mode, volume, pitch and envelope, the defaults too, and empties the pads without
  a file; test `Switch and switch back` heard (`switch_and_switch_back_nothing_leaks_into_the_sound`),
  render `renders/11-switch-nothing-leaks.wav`; commit.
- [ ] 6. Live check with the owner: the checklist from the proposal (roots, create, open,
  restart, switch, the deleted folder); what feels wrong goes to the spec first, then to
  code; commit if anything changes.
