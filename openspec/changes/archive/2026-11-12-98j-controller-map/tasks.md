# Tasks

## 1. Profile layout

- [x] 1.1 Parse `[layout]` and `[controls.crossfaders]` in `Profile::from_toml`. Report an invalid layout without a profile error.
- [x] 1.2 Move the X-Session crossfader to `[controls.crossfaders]`.

## 2. Short labels

- [x] 2.1 Give each action a short label of 1 to 6 ASCII cells. Test all actions.

## 3. Controller tab

- [x] 3.1 Add the `Controller` tab to F1: tab order, the device pages, the start device and hotplug.
- [x] 3.2 Build the view model of the drawing from the effective profile and `resolve_in`.
- [x] 3.3 Draw the cells, the selection line, the top device line, the list form and the empty state at 80x26 and 120x40.
- [x] 3.4 Move the selection with the arrows. Change the device with the help page actions on this tab.

## 4. X-Session layout

- [x] 4.1 Record each X-Session control with `aseqdump` while the owner presses them in order.
- [x] 4.2 Write the X-Session `[layout]` from the record. Check that it fits at 80x26.

## 5. Verification

- [x] 5.1 Cover each scenario with a test of the same name. Run `task check`.
- [x] 5.2 Add the owner checks to `docs/LIVE-CHECKS.md`.
