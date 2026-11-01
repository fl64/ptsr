# 04e — Chop keeps what lies on the pads

## Why

A chop is one file with cuts; pads play its slices. Today three things get in the way of "cut a slice, put it on a pad, chop on for other pads":

- Cuts made with no pad following them are lost on exit: the session is not saved after such edits.
- The assign mode always starts from slice 1, so the slice just heard has to be found again.
- A new cut inside a slice that already lies on a pad silently shortens that pad.

## What changes

- Every change of the cuts is saved, also when no pad follows them.
- Enter opens the assign mode with the selected slice, not slice 1. (Key moved by 21: the assign starts with `P`; `Enter` in listen opens the slice edit. Update the key in the scenarios when this change is implemented.)
- A new cut inside a slice that lies on pads unlinks these pads: they keep playing exactly what they played. The status names them; undo takes the cut back and links them again.
- An unlinked pad is a `pad` source (27): its panel says `amen.wav · pad`, the grid shows no slice number.
- The selected slice says where it lies: `slice 3 · 0.65 s · on 1Q, 2W`.

## Out of scope

- Slice parameters and their frozen copy on an unlinked pad (04b).
- Showing an unlinked pad's piece over the waveform.
- A command that puts the new slices in place of an unlinked pad.
- Saving slices as separate files (backlog, "save the edit").

## How to verify

1. Open a sample, F3, cut it with M in a few places, put nothing on pads, quit with Ctrl+Q and start again; F3 on the same pad: the cuts are there.
2. Space, listen, walk to slice 5 with `J` `L` (or `PgDn`), `P`: slice 5 is selected in the assign mode; hit Q, `Enter`: only slice 5 lies on Q.
3. With slice 1 on Q, cut inside slice 1: the status says pad Q no longer follows the cuts, Q's panel says `amen.wav · pad` and Q plays the whole old piece. Ctrl+Z: the cut is gone, Q is `slice 1` again.
4. Move the end of slice 1 when slices 1 and 2 lie on Q and W: the info line shows both pads, Q gets longer, W shorter.

Listen for: Q plays the same piece before and after a cut inside it.
