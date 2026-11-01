# 19 — Pad pitch and the change scope

## Why

The owner wants to tune pads like on the SP-404: the pitch and the volume of one pad, of a few pads, of a bank or of all pads at once. Today only the volume of one pad changes, and there is no pitch.

The sound settings live on the pad, not on the slice: the same slice on two pads can play at two pitches, and what a pad shows is what it plays. Envelopes (attack, decay, release) and the large slice view come next, in 04b rewritten for pads.

## What changes

- **Pitch of a pad**: −24…+24 semitones in steps of 1. Higher plays faster and shorter, like a record; no time-stretch.
- **Parameters on the pads screen (F2)**: `Tab` picks the parameter (volume, pitch), `-` `=` change it. The pad panel highlights the picked one.
- **Change scope**: what `-` `=` and the mode (`Enter`) act on. `Ctrl+A` steps it: pad → bank → all → pad. The panel always says it: `change: pad Q`, `change: 3 marked`, `change: bank 2`, `change: all pads`.
- **Marking pads**: `M` turns marking on and off. While it is on, a pad hit marks or unmarks the pad (it still sounds, so pads can be picked by ear); marked pads are framed in the grid. With marked pads the "pad" step of the scope becomes "marked". `Esc` drops the marks.
- **A group changes relatively**: pitch +1 adds a semitone to every pad in the scope, the differences between pads stay. A pad at a limit stays there; the pop-up says how many pads hit it. The mode is not a number: every pad in the scope gets the next mode of the selected pad.
- **Knobs**: the learned volume knob and a new `pad.pitch` knob set the selected pad with soft takeover; the other pads of the scope move by the same amount.
- **Session**: the pitch is saved per pad; sessions from before open with pitch 0.

## Out of scope

Cents, envelopes, filter, pan, undo of pad parameters (see design, open questions). The chop screen does not change.

## How to verify

1. F2, select a pad, `Tab` to pitch, `=` three times: the pad plays higher and shorter; a sounding pad glides, no click.
2. `Ctrl+A` until `change: bank 2`, pitch `-` twice: every pad of the bank is two semitones lower, a melody from slices keeps its shape.
3. `M`, hit three pads, `M`: they are framed; `change: 3 marked`; volume `-` four times: only they are quieter.
4. Restart: the pitches and volumes are back.
5. Listen to `renders/19-pad-pitch.wav`: one pad at 0, +12 and −12.
