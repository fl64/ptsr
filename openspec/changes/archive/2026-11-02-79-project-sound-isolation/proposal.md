## Why

An empty pad in a new project can play a sample or slice from the previous project.
The sound must match the open project.

## What Changes

- Keep the sounds and pad settings of each project separate, even during loading.
- Test the real command queue and a late sample load after a project switch.
- Fade out sounds from the old project when the owner switches projects.
- Do not change the project file format.

## Capabilities

### Modified Capabilities

- `projects`: empty pads and pending loads do not carry sounds across a project switch.

## Impact

Project switching, sample loading, the audio command queue and tests. No new dependency.

## How to verify

Put samples and slices on pads in a project. Create a new project with `Ctrl+P`.
Hit the same pads by keyboard and controller. They must not play the old sounds.
Switch back and make sure that the old project plays its own sounds. Listen to the
render for this change.
