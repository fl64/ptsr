# 57 — The learn mode shows the new control next to the old one and waits for Enter

## Why

The owner (2026-11-01), the learn mode flow:
"0. pick the element; 1. show its binding; 2. when I turn, show the new one in the same
place in another colour; 3. Enter / Esc confirm or cancel, Backspace removes it".
In 53 a free control binds at once and the pick moves on: the owner sees the result only
after the fact and cannot back out of a wrong knob.

## What changes

- A picked element shows its control where its label stands (or "no control").
- A moved control does not bind: it stands next to the old one, in the pending colour,
  until `Enter` binds it or `Esc` drops it. Another control moved before that takes its
  place.
- `Enter` binds and the pick moves on to the next element, as in 53.
- `Backspace` puts "no control" in the pending colour; `Enter` clears, `Esc` keeps.
- A control taken by another element in the same layer: that element lights in the
  "taken" colour and the line warns that it loses the control; `Enter` moves it. A control
  bound in another layer is named in grey, no warning.
- A second way in: do the action (a key, a pad hit, a value step), then `F10` within 5 s;
  the mode opens with that action picked, in its layer and page. The old way (`F10`, then
  the key, all silent) stays for actions with effects one does not want to make, such as
  deleting a cut.
- The learn mode key is `F10` in place of `F4`, next to the settings on `F12`.
- A parameter takes a knob, a command takes a button: the wrong kind is refused, and the
  top line says why in words.
- A binding goes to the layer where the action was picked: the edit group with the edit
  open (53 sent every binding to the screen's group); the dialog names the layer, so the
  owner sees where it goes.
