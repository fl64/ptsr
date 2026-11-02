## Why

A file name does not always tell the owner what a pad does. A pad needs a name that is
easy to recognize in a project.

## What Changes

- Give the selected pad a name with `Ctrl+N` on the pads screen.
- Save the name in the project or scratch session. A new sound gets its own file name.
- Keep sound playback and sample files unchanged.

## Capabilities

### Modified Capabilities

- `pad-playback`: name a pad and show its name.

## Impact

Pad state, session files, keyboard input, the pads screen and tests. No new dependency.
The optional name keeps old sessions readable. Renaming files and naming groups are out
of scope.

## How to verify

Put a sample on a pad, press `Ctrl+N`, enter a name and press `Enter`. Restart and check
the name. Replace the sound and check the new file name. Check `Esc` and the empty name.
The sound must stay the same when the name changes.
