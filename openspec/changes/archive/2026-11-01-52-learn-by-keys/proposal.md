# 52 — Learn by the element's own key, save only after a yes

## Why

The owner (2026-10-31) about the learn mode of 47: "absolutely unclear how to use it",
"I see something unclear at the bottom, how to read it, what to do so as not to break the
old", "only volume can be picked, how do I get to the filter?". Then the owner's own
design: "the keyboard keys stay; in F4 press a key, light the element, then move a MIDI
control, ask to save; light it especially if it was taken, and save only after a yes".

## What changes

- In `F4` the element is picked by its own key: a pad key picks the pad, a command key
  its command, `PgUp` `PgDn` the bank switch; `I` `K` walk the pad parameters, every one
  of them, the edit need not be open. The keys keep what they do outside the mode.
- A moved control is not saved at once: the top line asks "k5 → cutoff: save?"; when k5
  already drives something on this screen, that element lights too and the line says it
  will be replaced. `Enter` saves, `Esc` drops it.
- The top line of the screen says in words that the mode is on, what is picked and what
  to do next; the band of columns at the bottom goes away. Labels stand only on the
  elements that have a control.
- Every pad parameter can be learned on its own: a knob bound to the cutoff turns the
  cutoff on any page and outside the edit.
