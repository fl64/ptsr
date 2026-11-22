## Decision

This is the Panic part of planned 93a. The date-named first project is already in the projects specification. Redo is separate.

Use the existing Ctrl+Period reservation. Keep Panic outside the 16 panel cells, as change 134 requires. SMC-PAD uses two separate presses of its STOP button while its own command panel is open. The first press arms the action for 500 ms. It does not stop playback. Release PAD BANK, close the panel, or wait to cancel. With the hold shortcut off, first reveal the panel in the usual way.

At the audio boundary, apply Panic after the available commands for that block. Stop the clock and close the pattern pass at its current tick. Fade the active voices, click, and effect tails for at most five milliseconds. Clear wet buffers in bounded steps and play new dry sound during the clear. Preserve pad assignments, pattern content and sound settings. A microphone or output recording uses its normal finish path.

## Cases

| Situation | Result | Status |
| --- | --- | --- |
| Two different controller connections press STOP | One connection cannot complete the press of the other | Checked offline |
| One press repeats or stays down | It cannot run Panic | Checked offline |
| Panel closes, owner disconnects, or the interval expires | The next press starts a new gesture | Checked offline |
| Panic and queued hits reach one audio block | The block stops and fades the sounds before output continues | Checked offline |
| COUNT or a live pass is active | Cancel count-in or retain accepted hits | Checked offline |
| Old effects have energy and a new pad plays | Old wet sound goes silent. The new dry hit plays | Checked offline |
| Project switching overlaps the effect clear | The new project runs with clean effects | Checked offline |
| The owner uses ordinary STOP | The usual pause and recording routes apply | Checked offline |

## Open questions

- SMC-PAD gesture: use two STOP presses with the panel open. Use the default interval of 500 ms.
- Manual checks: use the targeted keyboard, SMC-PAD and render procedure after merge. The owner authorized this order.
