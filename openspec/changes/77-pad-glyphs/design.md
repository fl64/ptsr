# Design

## Decisions (owner, 2026-11-02)

- `⩘` (U+2A58) for the envelope.
- The mute group as a digit on a light plate (Q1.1).
- `↕` is checked by eye first; it changes only if it covers the frame (Q2.1). The
  owner's second screenshot (2026-11-02) showed `↕◣` as one blot, so `↕` changes too.
  The agent took `⇕` (U+21D5, width "N"), the same up-down arrow.
- The change goes into PR 2 together with 76 (owner: "do all this in pr 2"); it is still a
  change of its own in OpenSpec.

## Defaults taken by the agent

- The plate is dark text (`BG`) on bright text color (`BRIGHT`); on a hit flash the plate
  turns around (bright digit on the dark), so it stays seen on the white flash.
- The digit goes last on the frame, after the other glyphs, as `①` did.

## Open questions

- Does `⩘` keep the frame in the owner's Ghostty font? Default: yes; the owner checks it
  with `printf` and in the live check.
