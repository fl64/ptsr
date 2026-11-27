## Decision

Use the existing notice row for the tempo result. A warning has priority over the result. The transport publishes its existing pause flag in the current snapshot. The interface reads the hits at the start of the pass to select the record label. Put the state after the step position in the title. Keep the grid and recording quantize in the second row.

| State | Result | Status |
| --- | --- | --- |
| Second accepted tap | Publish the tempo notice. Keep the bank and step rows. | Add |
| Higher-priority warning | Keep the warning visible. | Preserve |
| Pause at tick zero | Show Paused, because the clock can resume that position. | Add |
| Empty A1 with a paused clock | Keep the record cue for empty A1. | Preserve |
| Second STOP | Show Stopped and the play cue. | Add |
| First pass gains hits | Keep First pass until the pass ends. | Add |
| Stop or resume while a frame is in flight | Use only the accepted snapshot. | Preserve |

## Open questions

Use bar:beat for the pause position. The owner gave no format preference.
