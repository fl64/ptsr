# 64 — The owner's keymap in the user config, over the shipped one

## Why

The owner (2026-11-01): "are all our hotkeys nailed down now?" The commands come from
`config/keymap.toml`, read relative to the folder the program starts in: started from
elsewhere, the shipped copy silently wins and the owner's edits are not seen. There is no
place of the owner's own for keys.

## What changes

- The shipped keymap is built into the program; the working folder no longer matters.
- `~/.config/arasaka-404/keymap.toml` (the user config folder) holds the owner's keys
  over it: a key there replaces the shipped binding of that key in its section; `"none"`
  takes a key away. Only what differs is written there, so a new shipped key arrives with
  an update.
- The pad keys stay for the pads: a command on them is refused, as in the shipped file.
- A broken file is named on the status line; the program starts with the shipped keys.
- 65 builds the F12 Keys tab on this file.
