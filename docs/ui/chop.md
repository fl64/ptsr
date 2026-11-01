# "Chop" screen (sketch)

**What already works (04a, 04c):** the wave in Braille dots (the trimmed part is dim), slice numbers, the selected slice brighter and with pink bounds, the playhead. Listen mode: the pads play the slices of the page (1–16, 17–32). A hit selects the slice. Assign mode (Enter): the real pads of the bank, the plan "→ 3", pink pads will be replaced, the bank row, the total "8 slices → banks 1–2 · replaces 2". By default it goes slice by slice. Tab selects "all in order" from the start pad. Before an overwrite, a dialog asks a question. The chop belongs to the file: F3 on a slice opens the whole break, and an edit of a cut changes the linked pads. F3 in the file browser opens the file without a pad. Knobs: the start and the end of the selected slice (k1, k2).

**Waits for 04d:** auto-chop by hits and by tempo, snap to hits (N).

**Waits for 04b:** a large view of the slice with the envelope, a pop-up parameter value, slice parameters (↑ ↓ − =), kitty graphics.

A sketch in characters, ~160×40. The real waveform is kitty graphics or Braille.

```
 F3 CHOP      break_lofi.wav   2.61 s · 92 bpm · 9 slices              marker snapped to a hit
──────────────────────────────────────────────────────────────────────────────────────────────
 1        2          3         4        5          6          7         8           9
 │▇▅▃▂▁   │▆▄▂▁      │▇▅▃▁     │▅▃▁     │▇▆▄▂▁     │▆▄▂▁      │▇▅▃▂     │▅▃▂▁       │▇▅▃▂▁
 │▇▅▃▂▁   │▆▄▂▁      ┃█▇▅▃▁    │▅▃▁     │▇▆▄▂▁     │▆▄▂▁      │▇▅▃▂     │▅▃▂▁       │▇▅▃▂▁
──────────────────────────────────────────────────────────────────────────────────────────────
 slice 03   0.548 → 0.809 s · 261 ms                                  +3 st   −2 dB   ◀ reverse
┌────────────────────────────────────────────────────────────────────────────────────────────┐
│ ░░░┃ ╱‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾╲      ┃░░░ │
│ ░░░┃╱ █▇▆▅▅▄▄▃▃▃▂▂▂▂▂▂▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁  ╲     ┃░░░ │
│    start  attack 2 ms                                                 fade 40 ms  end       │
└────────────────────────────────────────────────────────────────────────────────────────────┘
         ┌───────────────┐
         │  fade 40 ms   │   ← pops up for ~1.5 s when a parameter changes
         └───────────────┘
 pads — listen to a slice
 [1 01] [2 02] [3 03] [4 04]
 [Q 05] [W 06] [E 07] [R 08]
 [A 09] [S  —] [D  —] [F  —]
 [Z  —] [X  —] [C  —] [V  —]
──────────────────────────────────────────────────────────────────────────────────────────────
 PLAY [Space] play [M] cut   MARKER [← →] select [, .] move [Ctrl+, .] fine [Bksp] delete
 SLICE [↑ ↓] parameter [− =] value   AUTO [Tab] by hits [N] to hits ●   RESULT [Ctrl+Z] [Enter] to pads
 start with Tab (auto) or Space + M in time. Then adjust the markers and the slice   [F1] help
```
