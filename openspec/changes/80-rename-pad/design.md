## Context

A pad currently derives its displayed name from its sample file and slice. Pad state is
saved in TOML and is also used for undo and swapping.

## Goals / Non-Goals

Name a pad without changing its sound or its file. Do not rename files or groups.

## Decisions

Store an optional custom name with pad state. Missing fields in old sessions mean no
custom name. An older program can open the new file, but it cannot retain the new field
when it saves. A blank name removes the override. The name moves with the sound on swap
and clears only after a successful replacement or clear. Resetting parameters keeps it.

Use the existing text input pattern, but keep releases of keys held before the input
opened so gate notes do not get stuck. Keep input and state changes outside the audio
callback. A name change is an undo step.

## Risks / Trade-offs

A long or non-ASCII name can exceed a pad. Clip it to the available screen width without
changing the saved name. Test old session files, missing files, failed loads, swaps, undo,
keyboard layouts and session reload.

## Open Questions

Can an empty pad have a custom name? Default: no; tell the owner to put a sound on it
first. Can a custom name be applied to marked pads together? Default: no; rename only
the selected pad.
