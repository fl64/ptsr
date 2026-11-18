# Design

## Inputs and scope

Use the measured Master port map from [change 91](../archive/2026-11-16-91-smc-pad-profile/).
CC 30–37 select bank 1 roles. CC 38–45 select bank 2 roles.
The port has no separate KNOB BANK message. The owner reports that the knobs have
no detents. Use buttons for lists that have discrete pages.
Use the hardware positions for the strip, panel pair checks and controller tests.

Use the shared action path for each change. Use the same page data for routing
and for visible roles. Keep explicit owner bindings, including disabled controls.
The tables show the shipped roles. A binding can change the effective role.
During a question, a picker, text entry, Learn or Help, apply its input rules first.

## Bank 1 and the two list buttons

K1 means physical knob 1. The last column names the button pair in the help.
All other bank 1 roles stay dim when the page has no value for them.
K8 is Master on every page. The panel has its own More, Next and Main cells.

| Screen or mode | K1 | LEFT and RIGHT | Help label |
| --- | --- | --- | --- |
| Pads | Silent neighboring pad in the row | Previous and next pad bank | Bank |
| Pad edit | Parameter on the page | Previous and next pad edit page | Page |
| Pattern | Step in the selected row | Previous and next bar page | Bar |
| Pattern controls | Parameter on the page | Previous and next parameter page when available | Page |
| Chop and Chop edit | Cursor | Previous and next slice | Slice |
| Files | File or folder | Parent folder and enter selected folder | Folder |
| Settings | Row | Previous and next tab | Tab |
| FX | Parameter in the slot | Previous and next FX slot | Slot |
| Project picker | Project item | Existing picker route | Project |
| Paths folder picker | Folder item | Parent folder and enter selected folder | Folder |

Create the Pattern transport page in Pattern controls. The values use the
current tempo, Bars, Quantize and Strength handlers. Dim swing until it exists.
An isolated tempo movement uses 0.1 BPM. A fast turn can use a 1 BPM step.
Keep the existing A1 creation and Clear A1 confirmation rules.
On Files, RIGHT acts only if the selected item is a folder. The existing Load
and Take actions handle files. Use the existing bounds for lists and edit pages.
The K1 pad move stops at the row edge.

| Page | K2 | K3 | K4 | K5 | K6 | K7 |
| --- | --- | --- | --- | --- | --- | --- |
| Pads or Pad edit TONE | Mute group | Volume | Pan | Pitch | Fine tune | — |
| Pads or Pad edit ENV | — | Attack | Decay | Hold | Release | — |
| Pads or Pad edit FILTER | Type | Cutoff | Resonance | Envelope amount | Envelope decay | Key tracking |
| Pads or Pad edit SOURCE | Source | Pad region start | Pad region end | Loop start | Loop end | — |
| Pads or Pad edit FX | Drive | Send FX 1 | Send FX 2 | — | — | — |
| Pattern transport | Tempo | Quantize | Strength | Bars | Swing | — |
| Chop and Chop edit | Zoom | Slice start | Slice end | Chop region start | Chop region end | Threshold |
| Settings | Selected value | — | — | — | — | — |
| FX | Type | Slot value A | Slot value B | — | — | — |

A dash is an empty position. Dim the values that the sampler does not offer yet.
The Pads inspector and edit use the same page names and positions.
For the two FX slots, show the current effect names for value A and value B.
Follow the keyboard value bounds and apply the same Undo and safety rules.

For Chop, the K3 and K4 pair changes the selected slice. The K5 and K6 pair
changes the whole Chop region. Reuse the current relative bound steps and link
updates. Use four isolated steps on K2 for one former keyboard zoom step.
A turn in the opposite direction cancels pending zoom movement first.
Clear any pending movement when the source disconnects or the Chop closes.
Retain the relative acceleration rule for supported values and fast zoom turns.
Keep the one dB threshold comparison for slow turns. Move that role to K7.

## Bank 2

| Row | Left | Right |
| --- | --- | --- |
| 7–8 | Pan (dim until pad-sound) | Master |
| 5–6 | Send FX 1 | Send FX 2 |
| 3–4 | Pitch | Decay |
| 1–2 | Volume | Cutoff |

Use the selected pad on each screen. On Chop and Files, sample selection does
not change the pad target. Use the selected pad from the sampler state.
Keep the roles while the command panel shows. Follow existing overlay priorities.
Both master controls use the same action.

## Panel, tempo and deferred roles

Keep Cut and the existing region and marker actions on their panel routes.
Remove a value, parameter or editor page cell when a knob or button replaces it.
Show unused cells dim. Keep the keyboard and learn actions.
Mark and test the order of symmetric pairs in the page tables and panel map.
Use the physical grid width when checking pairs. A pair cannot cross a row.

Use tempo on the Pattern transport page. Remove the shipped global knob and
panel tempo steps. Keep tempo keyboard and MIDI learn actions.
Keep a dim Tap Tempo position until the action exists. When it exists, use the
`tempo` cell on the Pattern context route. Pan follows `pad-sound`.
When change 100 adds ptsr Note Repeat, use RATE, SWING, TEMPO and LATCH on bank 1
knobs 1–4 while it runs. The matching labels are on the device. This change does
not add Note Repeat, swing or the other missing sound parameters.

From the visible main page, More and the action cost two presses on page 1.
Each later context page adds one Next press. The PAD BANK switch and a missed
reveal add separate presses. See [the count table](../../../docs/CONTROLLER-PATHS.md).
Keep frequent actions on a direct route or the first context page.

## Existing saved assignments

`App::seed_device` copies shipped roles to saved bindings on first connection.
It does not store the origin of each entry. A reset discards the whole saved map.
Do not infer origin by equality with the old factory action.
Use the new layout for a new profile. For an existing profile, offer a review
before replacing saved roles. Show the prior and proposed role per affected slot.
Keep a personal or disabled binding. Ask the owner about an ambiguous old role.
Save only the accepted result. Cancel leaves the map as it was.
If persistence fails, keep the old effective map. A retry must not repeat a
partially applied update. Other device profiles and USB preset records keep their roles.

## Knob strip and help

Show two 2 by 4 grids side by side, top row 7–8 to bottom row 1–2.
Mark the last bank that sent valid knob movement on this connection.
Clear the mark on disconnect. Dim any inactive role. Show the effective page
and list name in the strip and the help summary row. Check 80 by 26 and 120 by 40.
The mark does not promise the physical switch position after a bank switch.

## Edge cases before implementation

| Case | Expected result | Status |
| --- | --- | --- |
| A new bank sends a turn before the next redraw. | Use the CC bank immediately. Show that bank on the next frame. | Support now with raw MIDI. |
| A question follows a knob or button event. | P7 and P8 answer once. No knob changes a value behind the question. | Preserve with input-order tests. |
| A held Gate crosses a bank or page change. | Its release reaches the original voice. | Preserve with raw MIDI. |
| A turn follows a stale disconnect event from the prior connection. | The old event cannot mark or change the new connection. | Preserve connection ownership. |
| One pair lies across a row boundary or the data reverses its roles. | The layout data check fails. | Support now with table tests. |
| A slow zoom turn reverses at a limit. | The reverse turn works without a jump. | Support now with controlled turn timing. |
| A saved factory value and an owner value look the same. | The owner sees the proposed replacement before approval. | Support now with persistence checks. |
| Another controller sends a turn during an SMC-PAD mode change. | Its current input route and hold stay active. | Preserve with two-device replay. |
| PAD BANK target flow needs a new count. | Measure target flows on the device before the later change. | Later panel change. |

## Open questions

- The first release can show dim controls for unavailable parameters. The
  default is to keep their assigned physical places for later changes.
- The layout update must expose its exact list of conflicting assignments.
  The default is to retain each old assignment until the owner resolves it.
- Target flow counts require the physical device. The later panel change
  will compare both routes before it selects one.
