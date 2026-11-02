# Design

## Decisions (owner, 2026-11-02)

- `⩘` (U+2A58) for the envelope.
- The mute group as a digit on a light plate (Q1.1), then, after the screenshot of the
  plate: the group moves to the mode line as `mute 2` (owner: "maybe show it here?",
  "ok").
- The mode on the pad as a glyph: `⇥` one-shot, `⇢` gate (the owner's pick over `⎍`).
- `↕` is checked by eye first; it changes only if it covers the frame (Q2.1). The
  owner's second screenshot (2026-11-02) showed `↕◣` as one blot, so `↕` changes too.
  The agent took `⇕` (U+21D5, width "N"), the same up-down arrow.
- The change goes into PR 2 together with 76 (owner: "do all this in pr 2"); it is still a
  change of its own in OpenSpec.

## Defaults taken by the agent

- The panel keeps the mode as a word; only the pad cell takes the glyph.
- No `·` before `mute`: in a 100-column window a pad has 13 cells inside, and
  ` ⇥ +1 · mute 2` took 14; ` ⇥ -24 mute 8` takes 13.

## Open questions

- Are `⇥` and `⇢` told apart at a glance in the owner's font? Default: yes; if not, gate
  takes `⎍`.
- Does `⩘` keep the frame in the owner's Ghostty font? Default: yes; the owner checks it
  with `printf` and in the live check.
