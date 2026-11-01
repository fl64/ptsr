# Design 57

## Decisions (owner, 2026-11-01)

- Steps 0–3 of the owner's message: pick, show the binding, show the new control in another
  colour in the same place, `Enter` / `Esc` / `Backspace`.

```
picked pad Q, bound to pad 5 of the controller:
┌Q── pad5 ──────────┐     top line: LEARN · pad Q · pad 5 · move a control

turned k3:
┌Q── pad5 → k3 ─────┐     k3 in the pending colour
                          top line: LEARN · pad Q · pad 5 → k3 · Enter bind  Esc keep

Backspace:
┌Q── pad5 → none ───┐     top line: LEARN · pad Q · pad 5 → none · Enter clear  Esc keep
```

- The owner's screenshot (2026-11-01): "show the change here, lit differently, so it is
  clear what is new", the arrow on the pitch row of the pad panel. A parameter shows the
  pending control in its own row, not only in the top line:

```
─ TONE ──────────────────────
1 volume  +0 dB  ◂ cc 14
2 pitch   -3 st  ◂ cc 24 → cc 26      cc 26 in the pending colour
```

- Then (the owner's own proposal, "ok" to the recommended): "I press a key or a combination,
  then F4; the dialog shows the action and what is bound to it; a knob or a button shows
  the new one next to it; I confirm or not; it finds the layer and the page by itself".
  The old way in stays (Q1.1); a knob on a parameter in the edit drives that parameter, not
  a page slot (Q2.1); `F10` catches the action within 5 s (Q3).
- The learn mode moves from `F4` to `F10` (the owner: "closer to the settings"; first
  `F11`, then `F10` after hearing that terminals take `F11` for full screen). The owner's
  words above still say F4. Some terminals open a menu on `F10` (GNOME Terminal does, from
  memory; Ghostty not checked): if the key never arrives, the live check finds it.
- "Show if the control is already taken on this layer": the taken element lights, the line
  warns; a binding on another layer is a grey line with no warning.

```
LEARN · cutoff · PADS · EDIT · FILTER
  now:  none
  new:  k5                          pending colour
  k5 is taken on this layer: resonance loses it
  Enter bind   Esc keep
```

- 53 sent every binding to the screen's group: the owner could not see where a binding
  went. Here the dialog names the layer, so the binding goes where the action was made.

- A button for a parameter is refused (Q1.1, "yes, with a clear explanation why"): a
  button that steps one parameter needed ~30 new actions; knobs turn parameters, buttons
  press commands, both ways refused with the reason in words.

- A knob bound to a parameter in the edit layer did not show on F12: the edit groups
  listed only the page knobs and the commands. Found after the code of 1a; the owner:
  "do it" (N3). The edit groups list the parameters too.

- The owner saw "edit pad needs a button, not a knob". The likely path (no log read):
  `F10`, then `Enter`, which in the mode picks the action of its key, "edit pad" (opening
  the edit with `Enter` and then `F10` picks nothing: the action was on the pads layer, the
  mode opens on the edit layer). "ok" to Q1.1: a refused control says how to pick
  another (`I` `K`, a pad, a command key); a pick by "do, then F10" says it is the last
  thing done.

## Defaults taken by the agent

- After `Enter` the pick moves on to the next element (53); after `Esc` it stays.
- The control just bound is not heard until another control moves (53), so a knob still
  turning does not land on the next element.
- `Backspace` waits for `Enter` as a bind does: one rule for every change.

## Archive order

"Learn on the screen" stands on 53: 57 is archived after 53.

## Open questions

- After `Enter` move on to the next element or stay? Default: move on. With the "do, then
  F10" way the mode closes after `Enter`: there is no next element to walk to.
- `Backspace` at once or with `Enter`? Default: with `Enter`.
