# Design 48

## Decisions (owner, 2026-10-31)

- Blocks by page, SOURCE last (the owner's own proposal).

```
┌ EDIT · TONE ENV FILTER SOURCE ┐
 pad F  bank 1
 rec-20261031-181756.wav · pad
 mode    one-shot
─ TONE ──────────
 volume  +0 dB
 pitch   +0 st
─ ENV ───────────
1 attack 0 ms
2 decay  0 ms
3 release 0 ms (gate)
─ FILTER ────────
 filter  off
 cutoff  1.0 kHz
 reso    0 %
─ SOURCE ────────
 source  pad
 start   3.244 s
 end     3.806 s
 change  pad F
```

## Archive order

"Parameter pages" and "The screen shows the layer" stand on 44: 48 is archived after 44.

## Open questions

- The panel grows by four lines. Default: the rows at the bottom are cut on a short
  terminal, as now.
