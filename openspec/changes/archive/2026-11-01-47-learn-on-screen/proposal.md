# 47 — Learn the controller on the screen itself

## Why

The owner (2026-10-31): "a mode to bind MIDI controls to what is on the screen: press a
key, then walk through ALL the elements that can have a control and attach controller
pads or knobs to them". Today a control is learned only from the list on the MIDI tab of
F12, far from the screen it acts on. Picked "Q1.1": as Ableton's MIDI Map
(assumption: from memory), the screen itself shows what can be bound.

## What changes

- `F4` turns the learn mode on and off on the pads, chop and files screens.
- Every element that can take a control carries a label with its control: pads, banks,
  the parameters of the pad panel, the commands of the screen.
- `J` `L` `I` `K` walk the elements, the picked one blinks; a moved control is bound to
  it and the pick goes on to the next element; `Backspace` clears the binding; `F4` or
  `Esc` leave. Pads are silent in the mode.

Details: design.md, open.
