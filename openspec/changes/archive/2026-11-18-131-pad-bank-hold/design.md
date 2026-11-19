# Design

## Owner decisions

On 2026-11-18 the owner asked for the panel on a PAD BANK hold instead of a toggle.
The owner chose hold only, without a toggle option, and a separate change after 127.
A visible question takes its No and Yes pads without a hold. The other pads stay
silent during a question, as for the keyboard pads.
The owner asked to see each hit on the panel. A hit cell lights for 120 ms, the
`HIT_FLASH` time of a pad hit. A dim or empty cell lights grey, so the owner sees
that the hit arrived but did nothing.

## Measurement

On 2026-11-18 a status poll every 52 ms gave 595 replies in 31 s. Three taps gave
pressed in 3, 2 and 2 replies in sequence. Two holds gave pressed without a gap for
2.1 s (42 replies) and 2.6 s (50 replies). The first poll after each release gave
released.

On 2026-11-18 ptsr read all 8 presets. Bank 8 is the PAD BANK bank. It has Note
pads in every preset except preset 2 (DAW, MCP pads). A third-party description of the
device reports that PAD BANK acts only during a hold on a Note bank and latches on an
MCP bank. Preset 5 has the factory pads. It differs only in the encoder records
that ptsr sets. The owner keeps preset 5.

## Pad routing

With the hold shortcut on, notes 36–51 and 52–67 of one physical pad mean the same
position. The hold state, not the note range, selects the cell or the pad. A release
goes to the target of its press. Without the shortcut, the current layer rule stays.

## Corner cases

| Case | Result |
| --- | --- |
| A hit arrives before the first pressed reply of a hold | The pad plays. The delay is at most 52 ms. |
| The owner releases PAD BANK while a cell pad is still down | The panel hides. The pad release runs nothing and plays nothing. |
| Copy or Take is in hand when the hold ends | The item stays in hand. A playing hit places it, as now. |
| A question opens during a hold | The No and Yes pads answer. The other cells do nothing until release. |
| A question is visible without a hold | The No and Yes pads answer. The other pads stay silent. |
| The status read fails or the preset changes during a hold | The program treats it as a release and hides the panel. |
| Help, a picker, text entry or learning opens during a hold | The panel hides. If the hold continues after the overlay closes, the panel shows again. |
| LEFT or RIGHT during a hold | They move the panel views, even with saved owner roles. Without a hold, they keep their screen roles. |
| Disconnect during a hold | The panel clears. Reconnect waits for a released status. |
| Two Master connections | A hold acts only on its own connection. |

## Open questions

- A hit before the first pressed reply plays the pad. The default accepts this delay of at most 52 ms.
- Each hold starts on the actions. The default does not remember the screens view between holds.
