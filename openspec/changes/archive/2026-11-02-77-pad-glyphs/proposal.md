# 77 — A quiet pad grid

## Why

The owner (2026-11-02, screenshots): the mute group `①` covered the corner of the pad
frame, `↕◣` ran together, and then, with all glyphs fixed: "it all looks like garbage, a
pile". Every pad told everything: the name, the mode, the volume, the group and up to
seven glyphs of settings, sixteen times over.

## What changes

- The grid shows what plays; the panel on the right shows how a pad is set. The way MPC
  and SP-404 show pads.
- The top frame of a pad carries no glyphs.
- The line under the name is the mode line: the mode as a glyph (`⇥` one-shot, `⇢` gate),
  the volume when not 0 dB, `mute N` for a mute group, and one mark `✱` when the pad has
  more settings (pitch, envelope, filter, drive, sends): `⇥ +1 mute 2 ✱`.
- F1 names `⇥`, `⇢`, `mute N` and `✱`.

## Out of scope

- A key that shows all the details on all pads for a moment.
- Lighting the pads of the same mute group when a pad is selected.

## How to verify

1. Look at the grid: no pad frame carries glyphs.
2. Load a sample on a fresh pad: the line under its name is `⇥`.
3. Put it into mute group 2 and lower it by 6 dB: `⇥ -6 mute 2`, no `✱`.
4. Raise its pitch: `⇥ -6 mute 2 ✱`. Turn the pitch back to 0: the `✱` goes.
5. Press `H`: `⇢ -6 mute 2`.
6. Press F1: it names `⇥` one-shot, `⇢` gate, `mute N` and `✱`.
