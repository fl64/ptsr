# 53 — The learn mode moves on after a binding and asks only when it must

## Why

The owner (2026-10-31): "the learn mode still does not really work, still unclear". The
log shows k10 bound to "next bank" eleven times in a row: after `Enter` the mode waited
for a control again, the knob was still turning, the question came back at once. The
binding landed in the edit layer, the pad edit was open, so k10 did nothing with the edit
closed. A knob was bound to a command, every tick of it would turn a bank. The owner:
"fix it, we will see", "once I picked and bound, pick the next element for me and do not
pester me with questions".

## What changes

- A free control is bound at once, no question; the pick moves on to the next element.
- The control just bound is not heard by the mode until another control moves.
- The mode asks only when the control already drives another element.
- A binding always goes to the screen's group, the edit open or closed.
- A command takes a button or a pad of the controller, not a knob; a knob turned for a
  command is not bound, the top line says a button is needed.
