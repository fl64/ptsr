# 70 — Mute groups

## Why

The owner (2026-11-01): "we need mute groups too, don't we?" They are in the first stage of
`docs/VISION.md` ("mute groups") and in the backlog (row 9). A drum kit needs them: a closed
hat cuts the open one, a new kick cuts the tail of the last. Now every pad rings out over
the others. The owner: "N2" (the agent's sketch, below).

## What changes

- Each pad has a mute group: off or 1–8. A hit of a pad stops the other sounding pads of
  its group with the short click-free fade a pad already uses to cut its own last hit.
- The group is the third parameter of the TONE page (knob 3), after volume and pitch; it
  follows the change scope, so `M`, three pads, `M` and group 1 put all three in it.
- Groups are shared by all banks: pad 1-1 and pad 2-1 in group 1 cut each other.
- The pad frame wears the group as a circled digit `①`…`⑧`; F1 explains it.
- The group is saved with the pad in the project; an older project opens with groups off.
- `Ctrl+Z` undoes a group change, `Ctrl+X` swaps the group with the rest of the pad.
