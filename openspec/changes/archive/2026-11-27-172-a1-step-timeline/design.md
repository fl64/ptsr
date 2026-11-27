## Selected time

The overview and Step Edit use the same musical position. When the owner
has not chosen a cell in the overview, use the playback cell or step one.
After the owner moves the cursor, the page follows the selected step.
Playback uses a separate mark and keeps its running position.

Keep the existing three-character grid cells. Show the global count
above the grid. Use the existing beat and bar separators to align the
time labels with the pad rows.

```text
STEP EDIT · A1 · Bar 2/2 · Step 17/32
Grid 1/16 · 16 per bar · Record quantize 1/8 75%
```

## Input routes

Left/Right and J/L are the shipped keyboard commands for step movement.
Use the resolved actions so custom bindings and `none` still apply.
Keep the SMC-PAD STEP encoder route. Move one cell for one step action.
Keep the active question, Help, and Setup input priorities.

F7 and command pad 6 enter Step Edit. F7, Esc, and command pad 1 leave it.
Enter on the selected time. Return the edited time to the overview.
Changing the grid keeps the same musical position in the new grid.

The owner can add or remove one hit with a pad in Step Edit. Keep Undo
on Ctrl+Z and command pad 2. Keep the existing check for multiple hits
that share a cell. The status line gives the name of the edited pad,
the global step, and the result.

## Checks

| State | Expected result | Check |
| --- | --- | --- |
| No A1 | The overview shows its empty state and keeps the edit refusal | Keyboard and raw MIDI entry test |
| Empty A1 | A valid first step shows as the selected column | Frame at both sizes |
| End of bar | Step 16/32 becomes step 17/32 | Keyboard and raw MIDI navigation test |
| Playback in another bar | The selected view remains stable and playback has a separate mark | App frame with a live transport snapshot |
| A finer grid or a shorter loop | The selection follows the musical position or clamps to the last visible cell | Frame and input test |
| A question or Setup owns input | The time selection does not move behind that screen | Input test |

## Scope

Read and edit one step position. A later recording check covers the
count-in and live pass prompts.
