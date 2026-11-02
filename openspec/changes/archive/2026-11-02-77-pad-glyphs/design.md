# Design

## Decisions (owner, 2026-11-02)

- The mode on a pad as a glyph: `⇥` one-shot, `⇢` gate (the owner's pick over `⎍`).
- The mute group on the mode line as `mute N` (not on the frame: `①` covered the corner,
  a digit on a plate looked like one more glyph).
- No glyphs of the settings on the pad: one mark `✱` at the end of the mode line says
  that the pad has more settings (owner: "maybe show that there are some other settings
  on the pad, except the source?", then "ok" to `✱`).
- The change goes into PR 2 together with 76 (owner: "do all this in pr 2"); it is still a
  change of its own in OpenSpec.

## Defaults taken by the agent

- The panel keeps the mode as a word; only the pad cell takes the glyph.
- No `·` before `mute`: in a 100-column window a pad has 13 cells inside;
  ` ⇥ -24 mute 8 ✱` takes 13.
- `✱` counts what the mode line does not show: pitch, envelope, filter on, drive, sends.
  Not the volume, the mode, the group (on the line already) or the source (a slice
  shows in the name).
- The requirement "The pad shows what is set on it" (55) is removed, and "The pad marks
  its settings" takes its place: its scenarios about frame glyphs no longer hold.

## Open questions

- Are `⇥` and `⇢` told apart at a glance in the owner's font? Default: yes; if not, gate
  takes `⎍`.
