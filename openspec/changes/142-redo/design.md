## Decision

Use the state that Undo replaced as the Redo step. Restore it through the same audio and pad paths as Undo. Keep the project file format.

The latest successful Undo sets the next Redo step. Each Redo recreates an Undo step. With a different edit context active, Redo asks the owner to return to the applicable context. For a completed active-pass Undo, retain the accepted take when the collector restores the pre-pass pattern. Read the audio boundary before redo.

The target view already uses P3 to open its action menu. Keep this route. In the menu, P3 runs Redo and P1 returns to targets. P1 on the target view still cancels. Modal questions and text input have priority.

## Cases

| Situation | Result | Status |
| --- | --- | --- |
| Undo a grouped pad or FX edit, swap, or load | One Redo restores the same state and pad addresses | Pad, swap, and regression tests pass |
| Start a different edit, or change project | Old redo steps clear | New edit and project tests pass |
| Move around with an undone edit | Redo remains available in its context | Playback and Chop context tests pass |
| Undo an active A1 pass, then redo | Restore accepted rows after the pass closes | Active and closed pass tests pass |
| Undo Chop bounds, recut, normalization, or a fresh Chop | Restore the edited Chop and affected pad state | Cut, normalization, and fresh Chop tests pass |
| Undo an Assign choice | P3 Menu then P3 Redo restores it | Keyboard and controller tests pass |

## Open questions

None. The owner deferred manual checks until after merge.
