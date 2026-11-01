# Design — 13

## What the owner sees

On the pads screen, between the bank row and the grid, a **3-row live strip** of the master output: a rolling envelope where what plays right now enters at the right edge and drifts left, so a hit shows as a wave running across the strip. The strip is drawn **in decibels on the same scale as the meters** (floor −60 dB = the middle line, 0 dBFS = full height): a −6 dB peak reads as tall on both, not tall on the bar and half-height on the wave. Silence is a thin middle line, and the picture does not move while nothing plays. At the right of the strip, in the top 3 rows of the info column: **L/R meters** of the master output, one row per channel — a bar in dB, a peak-hold mark, a latching overload light.

The first take on the strip (static per-pad sample shapes) missed the owner's words "I play — I see what I play": the strip is the **live output**, not a catalogue of samples. Static shapes are gone.

## Where the numbers come from

- **Output history**: `engine::rt::Stats` keeps a ring of recent per-block output peaks — `[AtomicU32; 1024]` of f32 bits (max of |L|, |R| per block) with an atomic cursor, written from the same place the peaks are recorded. At 128 samples / 48 kHz that is ~2.7 s of history; no allocation, no locks — a fixed ring and `fetch_add` on the cursor. The UI reads a snapshot each frame (torn values at the write position are fine for a picture).
- **Peaks for the meters**: `out_l`/`out_r` atomics with the f32-bits `fetch_max` trick, reset by `take_output()`. The offline render path records the same way, so a test can prove the meters agree with the rendered WAV.
- **Peak hold, decay, clip latch** are UI-side state in the app (never in the audio thread): each UI tick reads `take_output()`, converts to dB (20·log10), the bar falls about 40 dB/s, the peak mark holds ~1.5 s and then follows the bar down. A block peak ≥ 1.0 latches the overload light; the next pad hit whose blocks never reach 1.0 clears it — the light then answers "did the last thing that played clip".

## Colors

The meter bar uses the palette's green (the bank-4 tone `#86C47E`); the live strip is drawn dim like other overview marks; the overload light is magenta — the style's attention color, already meaning "look here". A true red would break the two-accent rule; if the owner wants a real red for clips, that is a palette question (see open questions).

## Layout

`pads()`: vertical `[bank row 1, strip 3, grid fill]`; the strip splits `[grid width, gap 3, fill]`; the fill column splits vertically `[meters 3, pad info rest]`. The strip and meters live only on the pads screen.

## Open questions

1. **Overload light reset.** Taken: clears on the next hit that does not clip. Alternative: hold until the owner clears it with a key.
2. **True red for clips.** Taken: magenta. The owner asked for "green with red" — if magenta does not read as alarm, add one red to the palette.
3. **History length.** Taken: ~2.7 s (1024 blocks at 128/48 kHz). Longer history = slower drift.
