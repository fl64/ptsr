# 74 — Learn on F12 shows what it changes before it saves

## Why

On the MIDI tab of F12 a moved control is assigned and saved at once, blind. The owner (2026-11-01): before the save, the learn must show what the control will change, on F10 and on F12. A control learned in one group keeps its actions in other groups (the layers, chosen by the owner: one knob turns the volume on the pads screen and the cutoff in the pad edit). The spec scenario "Reassign a button" said the opposite and is fixed.

## What changes

- F12: the moved control stands pending on the row; the row of the same group that loses it lights in the taken colour; actions in other groups that keep it show in grey with `stays`. `Enter` saves, `Esc` changes nothing.
- F10 already shows this before `Enter` (57) and does not change.
- "One control, one action": per group; the other groups keep the control.
