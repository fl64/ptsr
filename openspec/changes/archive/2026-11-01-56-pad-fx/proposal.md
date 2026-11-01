# 56 — Drive on every pad, two shared effect slots fed by sends

## Why

The owner (2026-11-01): "let's add delay, reverb and distortion". Then: "on a pad, how many
percent I send and to which FX, and the FX themselves are any". This is the return-effect
way of MPC and DAWs (from memory, not checked) and the "buses 1 and 2 (send)" of
`docs/VISION.md`, section 10.

## What changes

- Every pad gets a drive (0–100 %, off by default) and two sends: to FX 1 and to FX 2
  (0–100 %, 0 by default). With all three at 0 the pad sounds as before.
- FX 1 and FX 2 are two effect slots shared by the whole project. Each holds any effect of
  the list: reverb (size, tone) or delay (time, feedback) for now. A new project has a
  reverb in FX 1 and a delay in FX 2. Their tails sound on after the pad stops.
- The pad edit gets three more pages: FX (drive, send 1, send 2; this pad), FX1* and FX2*
  (the slots, shared by all pads: knob 1 the effect type, knobs 2 and 3 its parameters).
  `Tab` walks them as the other pages (54).
- The pad frame glyphs (55) get `↯` for the drive and, for every send above 0, the glyph of
  the effect in that slot: `◌` reverb, `⋯` delay.
- Saved in the project; an older project opens with the drive and the sends at 0.
- Offline renders in `renders/` for the owner.

## Not in this change

- More effect types, more slots, chains in a slot, effects on the master or the input.
- The delay time in beats (no tempo yet), a tone knob for the delay.

## Dependencies

None new: the drive, the delay and the reverb are small own code in `engine`.
