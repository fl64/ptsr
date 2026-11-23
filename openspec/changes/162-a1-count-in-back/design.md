## Owner decision

Esc and command P1 cancel an active count-in and keep the A1 overview open. The owner approved this behavior after the manual A1 review. The first press closes Help or a question if it covers A1.

## Boundaries

| State | Expected result | Check |
| --- | --- | --- |
| Count-in on A1 | Cancel through the current pass command. Keep the bank, pad and pattern. | Keyboard and raw MIDI |
| Help or a question over count-in | The first press closes the top window. Count-in continues. | Keyboard and raw MIDI |
| P1 in the playing layer | Play the pad and keep count-in active. | Raw MIDI |
| Hold panel off, panel hidden | Command P1 cancels without panel reveal. | Raw MIDI |
| Live pass or idle on A1 | Back follows the existing route. | Existing mode and capture checks |

## Open questions

None.
