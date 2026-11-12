# Proposal

## Why

The owner plays from a MIDI controller and does not remember what each control does on each screen. The F12 MIDI tab shows a table of bindings, not the device. The owner asked for a picture of the device top with the action of each control. The plan for this picture is in `docs/plans/CROSS-CUTTING.md`, section 3.5.

## What Changes

- A device profile can have a `[layout]` section. It gives rows of control names in the order of the device top. A new group `[controls.crossfaders]` marks a crossfader.
- F1 gets a third tab, `Controller`. It draws the top of one connected device. Each control shows a short label of its current action on the screen under F1. A free control shows `-`.
- On the `Controller` tab, the arrows select a control. The line under the drawing names the control, its kind, its action and the action's full help text.
- With more than one device, the help page keys (Alt+PgUp, Alt+PgDn) change the device. A top line names all connected devices and marks the current one. The tab starts on the device that sent the last event.
- A profile without `[layout]`, or with a layout that does not fit, shows a list of control and action.
- Without a connected device, the tab says so and names the known profiles.
- Each action gets a short label of not more than 6 cells.
- The X-Session Pro profile gets its `[layout]` after the owner gives the place of each control.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `screen-help`: the F1 `Controller` tab, its selection and its device pages.
- `input-devices`: the `[layout]` section and the crossfader group in device profiles.

## Impact

- `crates/input`: profile parse of `[layout]` and `[controls.crossfaders]`, short labels of the actions.
- `crates/app`: the F1 tab state, the device pages, the view model of the drawing.
- `crates/tui`: the drawing of the tab.
- `config/devices/xsession-pro.toml`: the crossfader group and the layout.
- No change to sound, to the routing of MIDI events or to the user config format.

## Out of scope

- The live map on the F12 MIDI tab (a press lights its control, Enter learns it). A later change.
- The SMC-PAD layout, its knob banks and the STOP panel cells. Change 91 measures the device first.
- The printable `docs/SMC-PAD.md` page.

## How to check

1. Connect the X-Session Pro. Open F2 and press F1. Press Tab twice to open `Controller`.
2. Check that the drawing shows two decks, four faders and the crossfader in their places on the device.
3. Turn to the Chop screen (F3) and open F1 `Controller` again. Check that the knob labels changed to the Chop actions.
4. Move the selection with the arrows. Check that the line under the drawing names each control and its action.
5. Disconnect the device while the tab is open. Check the text `No MIDI device is connected` and the list of known profiles.
