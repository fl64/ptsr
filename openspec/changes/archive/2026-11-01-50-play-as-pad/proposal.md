# 50 — The chop screen plays a slice as its pad does

## Why

The owner (2026-10-31): "in the editor a slice plays without the settings of its pad;
maybe make it optional". The chop screen plays the bare sample: the pitch, volume,
envelope and filter of the pad the slice lies on are not heard. Picked "Q1.1".

## What changes

- A switch on the chop screen, `Alt+P` ("as pad"), off when the chop screen opens: on,
  what `Space`, the cursor and a slice pick play sounds with the settings of the pad the
  slice lies on (the first such pad); a slice on no pad plays bare.
- The help line shows `as pad` next to the play mode while it is on.
