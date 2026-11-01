# Design 56

## Decisions (owner, 2026-11-01)

- A drive per pad; two shared slots fed by a send knob per pad; any effect type in a slot
  (the owner's own proposal, after "one reverb only?").
- Two slots, as `docs/VISION.md` has them; two reverbs mean a reverb in both slots.
- The slots are set in the pad edit, on pages marked with a star, no separate screen.
- Glyphs: `↯` drive, `◌` reverb, `⋯` delay.
- The delay has no tone knob: the page holds three knobs, the type takes one ("go" to the
  recommended).

```
 pad ──[drive]──┬───────────────────────► out
                ├─ send 1: 20 % ─► [FX 1: reverb] ─► out
                └─ send 2: 50 % ─► [FX 2: delay ] ─► out

EDIT · TONE ENV FILTER SOURCE FX FX1* FX2*
─ FX ────────────────────    this pad
1 drive     40 %
2 send 1    20 %  reverb
3 send 2    50 %  delay
─ FX 1 * shared ─────────    one for all pads
1 type      reverb
2 size      50 %
3 tone      50 %
─ FX 2 * shared ─────────
1 type      delay
2 time      250 ms
3 feedback  40 %
```

## Defaults taken by the agent

- Ranges and steps: drive, sends, size and tone 0–100 %, 1 % a tap, 10 % a held key; delay
  time 10–2000 ms, 10 ms a tap, 100 ms a held key; feedback 0–90 %, so the echo always
  dies out. The type steps through the list.
- The drive keeps the loudness about the same: turning it up adds grit, not volume.
- A type change while the slot sounds: the old tail fades out over a few milliseconds, the
  new effect starts empty; each type keeps its own parameters in the slot, so a type
  turned back finds them as they were.
- The FX page follows the change scope (marked pads) as the filter does; the slot pages
  ignore it, the "change" line says "all pads" there.
- The slots live in the project, next to the pads.

- The panel height (found at step 3): all seven blocks at once take ~34 rows, a low
  window cuts the bottom. Mockups did not help the owner choose ("not clearer"); the agent
  takes the default and the owner judges it live: the FX block always (drive, send 1,
  send 2, each send names the kind of its slot), the block of a shared slot only while the
  edit is on its page. This replaces the mockup above where FX 1 * and FX 2 * stand always.

- The live check of the agent (2026-11-01, a window of 52 rows): on FX1* the slot block
  under the FX block fell below the window, only its line was seen; the frame title lost
  `FX2*`. The slot block now takes the place of the FX block on its page (the sends name
  the slot kind there anyway), and the title drops `EDIT ·` when it does not fit ("yes" to
  the proposal).

## Archive order

"Parameter pages" stands on 54, "The pad shows what is set on it" on 55, "The screen shows
the layer" on 48: 56 is archived after 55.

## Open questions

- Delay time in beats: waits for a tempo. Default: milliseconds.
