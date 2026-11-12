# Design

## Decisions

- **The layout is data.** A profile gets this section:

  ```toml
  [layout]
  rows = [
    ["k1", "k4", "", "k7", "k10"],
    ["b1", "", "crossfade", "", "b2"],
  ]
  ```

  Each string is a control name from `[controls]` of the same profile. An empty string is an empty place. The group of the control in `[controls]` gives its kind: `pads` is a pad, `buttons` is a button, `knobs` is a knob, `faders` is a fader, `crossfaders` is a crossfader. The parse reads `crossfaders` as it reads `faders`. Other programs that read the profile do not change.
- **Layout errors do not stop the device.** An unknown name, a name two times, or an empty `rows` makes the layout invalid. The device keeps its controls and actions. The status shows the file and the error, as for other profile errors. The tab shows the list.
- **Cells.** The drawing stands in the middle of the frame. Each place is 8 columns and 2 rows. Row 1 is the control name in dim text. Row 2 is the action label in brackets of its kind: knob `(volume)`, button `[volume]`, pad `|volume|`, fader `=volume=`, crossfader `<volume>`. All glyphs are ASCII. A free control shows `-` in dim text. A binding to an unknown action does nothing ("Every binding acts"), so it shows `-` too.
- **Fit.** The drawing must fit inside the F1 frame. If a row is too wide or the rows are too many for the frame, the tab shows the list for that device. The X-Session layout must fit at 80x26.
- **The label is the effective action.** The label comes from the same profile object and the same resolution that the MIDI dispatch uses (`Profile::resolve_in`). It uses the screen under F1, the edit layer when an edit is open, and the learned bindings over the profile. Thus the drawing cannot show an action that a press does not do.
- **Control order.** The TOML parse keeps no file order. The list and the controls of a profile go by kind (pads, knobs, faders, crossfaders, buttons), then by name with the numbers in order: `p2` before `p10`.
- **Short labels.** One function in `crates/input` gives each action name a short label: ASCII, 1 to 6 cells. It covers the names of `ACTIONS` and the knob parameters. A test checks every name. The full `label` stays for the help line.
- **Tabs.** F1 has the tabs `Help`, `Messages` and `Controller`. Tab goes to the next tab and Shift+Tab to the previous. Each F1 opening selects `Help`, as now.
- **Selection.** On `Controller`, Up, Down, Left and Right move the selection to the nearest control in that direction. Empty places are skipped. The list form moves up and down. The line under the drawing gives `name · kind · action · full label`. The first selection is the top left control.
- **Device pages.** The help page actions (`help.page_prev`, `help.page_next`, Alt+PgUp and Alt+PgDn by default) change the device on `Controller`. On `Help` they keep their key table paging. After the last device comes the first again. The top line names each connected device. The current device is bright. Two devices of one profile show as `X-Session Pro 1` and `X-Session Pro 2`, in the order of connection.
- **One device, several ports.** One device can connect with several ports. The X-Session Pro has `USB X-Session MIDI 1 20:0` and `USB X-Session MIDI 2 20:1`. The ALSA client number (`20` before the colon) identifies the device. All ports of one client with one profile are one page. A port name without this number is a device of its own.
- **Start device.** Each time the tab is selected, it shows the device that sent the last MIDI event. Without such an event, the first connected device.
- **Hotplug.** A device that disconnects while its page is open is removed. The tab then shows the next connected device, or the empty state. A device that connects while the tab is open gets a place on the top line. The current page stays.
- **Empty state.** `No MIDI device is connected.` and then `Known profiles:` with the names of the loaded profiles. The `.example` profiles are not loaded and are not named.
- **The X-Session Pro layout.** The owner presses each control in a fixed order while the agent reads the MIDI messages with `aseqdump`. The agent writes the places from this record only.
- **The X-Session record (2026-11-12).** Deck A is on the left, deck B on the right. The outer knob column is 1, 2, 3. The inner column is HIGH, MID, LOW.

  | Place | Deck A | Deck B |
  | --- | --- | --- |
  | Knob 1, 2, 3 (outer, top down) | `k1` `k2` `k3` (CC 24–26) | `k10` `k11` `k12` (CC 34–36) |
  | HIGH, MID, LOW (inner, top down) | `k4` `k5` `k6` (CC 27–29) | `k7` `k8` `k9` (CC 31–33) |
  | Headphone button | `b2` (note 44) | `b3` (note 45) |
  | LEFT, RIGHT | `b4` `b1` (notes 46, 43) | `b5` `b6` (notes 56, 57) |
  | PLAY/PAUSE, LOAD | `b10` `b7` (notes 70, 58) | `b9` `b8` (notes 69, 59) |

  The faders from left to right are `fader_a` (CC 12), `fader_b` (CC 11), `pitch_a` (CC 14) and `pitch_b` (CC 15). The crossfader is `crossfade` (CC 17). It also sends CC 20, which the profile ignores.
- **The X-Session layout.** Nine places in each row, so the drawing fits at 80x26:

  ```toml
  [layout]
  rows = [
    ["k1",  "k4", "", "", "", "", "", "k7", "k10"],
    ["k2",  "k5", "", "", "", "", "", "k8", "k11"],
    ["k3",  "k6", "", "", "", "", "", "k9", "k12"],
    ["",    "b2", "", "", "", "", "", "b3", ""],
    ["",    "", "fader_a", "fader_b", "", "pitch_a", "pitch_b", "", ""],
    ["b4",  "b1", "", "", "", "", "", "b5", "b6"],
    ["b10", "b7", "", "", "crossfade", "", "", "b9", "b8"],
  ]
  ```

## Risks

- About 200 actions need a short label. It is a large table edit. A test catches a missing or long label.
- The list form must stay readable at 80x26 for a profile with many controls. It scrolls with the selection.

## Open questions

- Are the four faders in the right order on the drawing? The record gives `fader_a`, `fader_b`, `pitch_a`, `pitch_b` from left to right. The plan photo names them PITCH A, VOLUME A, VOLUME B, PITCH B. Default: the record. The profile names may not match the print on the device.
