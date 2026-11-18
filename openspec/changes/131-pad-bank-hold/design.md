# Design

## Owner decisions

On 2026-11-18 the owner asked for the panel on a PAD BANK hold instead of a toggle.
The owner chose hold only, without a toggle option, and a separate change after 127.
A visible question takes its No and Yes pads without a hold.

## Measurement

On 2026-11-18 a status poll every 52 ms gave 595 replies in 31 s. Three taps gave
pressed in 3, 2 and 2 replies in sequence. Two holds gave pressed without a gap for
2.1 s (42 replies) and 2.6 s (50 replies). The first poll after each release gave
released. The device switches its pad layer on each press.

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
| A question is visible without a hold | The No and Yes pads answer. The other pads play. |
| The status read fails or the preset changes during a hold | The program treats it as a release and hides the panel. |
| Help, a picker, text entry or learning opens during a hold | The panel hides. If the hold continues after the overlay closes, the panel shows again. |
| LEFT or RIGHT during a hold | They move the panel views. Without a hold, they keep their screen roles. |
| Disconnect during a hold | The panel clears. Reconnect waits for a released status. |
| Two Master connections | A hold acts only on its own connection. |

## Open questions

- A hit before the first pressed reply plays the pad. The default accepts this delay of at most 52 ms.
- Each hold starts on the actions. The default does not remember the screens view between holds.
