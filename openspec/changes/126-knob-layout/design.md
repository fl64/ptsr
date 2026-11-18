# Design

## Owner decisions

The owner made these decisions on 2026-11-18 after a test of changes 124 and 125:

- Pad values change only in pad edit. Knob 1 selects the page there.
- The value knobs follow the screen order from top to bottom.
- Bank 2 has no pad roles.
- Relative knobs have no acceleration. The owner will ask for it again if necessary.
- The panel cell numbers follow the device labels. A separate change does this.

## Physical order

The SMC-PAD rows are 7–8, 5–6, 3–4 and 1–2 from top to bottom. A pad edit page
shows its values in one column from top to bottom. The left column of knobs
gives them the same order.

| Mode | K1 | K3 | K5 | K7 | K2, K4, K6 | K8 |
| --- | --- | --- | --- | --- | --- | --- |
| Pads | Silent pad selection in the row | — | — | — | — | Master |
| Pad edit | Page | Value 3 | Value 2 | Value 1 | — | Master |

A dash is an empty position. An empty position changes nothing.

| Page | Value 1 (K7) | Value 2 (K5) | Value 3 (K3) |
| --- | --- | --- | --- |
| TONE | Volume | Pitch | Mute group |
| ENV | Attack | Decay | Release |
| FILTER | Filter | Cutoff | Resonance |
| SOURCE | Source | Start | End |
| FX | Drive | Send FX 1 | Send FX 2 |

The values come from the page list in the program. A later page value gets the
next free knob in the left column only after a spec change.
The knob uses the same value target as the keyboard value keys in pad edit,
including marked pads. Knob 1 uses the existing page action and its bounds.
LEFT and RIGHT keep their current routes. The other screens keep their roles.

## Bank 2

Bank 2 knobs 1–7 have no shipped role. Knob 8 keeps master volume.
The global send pair leaves the pair table.

## Saved maps

The profile copies its roles into the user config on the first connection.
The saved map of the owner has the roles of change 124. The existing layout
review compares a saved slot with known old roles. Add the change 124 roles of
bank 2 knobs 1–6 as known old roles. The proposed role is empty.
The Pads slots keep the `smc.page_2` to `smc.page_7` actions. The program gives
them the new meaning, so these slots need no update.

## Relative steps

Each relative step applies its decoded count with multiplier 1.
The program removes the 40 ms speed history.
The Pattern tempo knob used 0.1 BPM for a slow step and 1 BPM for a fast turn.
Without acceleration it uses the general rule: 1 BPM per step, 0.1 BPM in fine mode. The fine button keeps its action and release rules. It has no
visible effect while acceleration is absent. Chop zoom keeps its quarter step.

## Knob strip

Mark the knob that sent the last valid movement, with its bank.
While the value pop-up shows, the strip help row shows the knob, value name,
target and value. The existing pop-up stays in place.
A neutral value, an empty position or a disconnect does not mark a knob.

## Edge cases

| Case | Expected result | Status |
| --- | --- | --- |
| A knob turns on Pads before pad edit opens. | No pad value changes. | Test with raw MIDI. |
| A page has fewer than three values. | The empty knob changes nothing. | Not shipped now. Table test. |
| Marked pads exist in pad edit. | The knob follows the keyboard value scope. | Test with raw MIDI and keys. |
| A saved map has a personal bank 2 role. | The review keeps it. | Test persistence. |
| A turn arrives during a question or Help. | The overlay keeps priority. | Preserve current tests. |
| A held Gate crosses a page change. | Its release reaches the original voice. | Preserve current tests. |

## Open questions

- Chop zoom without acceleration needs many turns for a long sample.
  The default is to keep the quarter step until the owner asks for a change.
- The tempo knob changes 1 BPM per step. The default gives tempo a usable range
  without acceleration. Ctrl+Shift+Up and Ctrl+Shift+Down keep the 0.1 BPM steps.
- The fine button has no effect without acceleration. The default is to keep it
  for a later acceleration change.
