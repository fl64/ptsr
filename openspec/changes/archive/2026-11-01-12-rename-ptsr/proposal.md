# 12 — The program is named ptsr

## Why

The project name is ptsr (PT's SampleR). The code still uses the working name `arasaka-404`: the program file, the folders of the owner's settings, the sound and MIDI clients. The docs say ptsr, so the paths in the docs do not work. The repository moves to `~/git/fl64/ptsr`.

## What changes

- The program file is `ptsr`: `cargo build -p ptsr`, `target/release/ptsr`.
- The folders are `~/.config/ptsr/`, `~/.local/share/ptsr/`, `~/.local/state/ptsr/`.
- At the first start, ptsr copies the config and the session from the old `arasaka-404` folders. The old folders stay.
- The sound server and MIDI see the client `ptsr`. Links made by hand to `arasaka-404` must be made again.
- The window title, screen texts, comments and test folders say `ptsr`.
- The docs name the new paths.

## Out of scope

- The sound reference "Access To Arasaka" (an artist name) stays.
- The `~/ptsr/` projects and samples roots (change 11) do not change.
