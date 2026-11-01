# 13 — Session overview: see what plays

## Why

Sixteen pads look alike whether they hold a quiet hats loop or a loud bass drop, and nothing shows how loud the output is right now. The owner asked twice: "the envelope in three rows of what plays — I see what I play" and "equalizers jumping with decibels, showing overload".

## What changes

- **Bank envelope strip** on the pads screen above the grid, 3 rows tall: one slot per pad of the open bank, the slot shows the peak envelope of the pad's sound; an empty pad is a thin middle line.
- **L/R output meters** at the right of the strip: the master output level in dB per channel, a peak-hold mark, and an overload light that latches when the output clips.
- The engine reports output peaks per channel through lock-free atomics; the loader computes the peak envelope of a sample once at load.

## Out of scope

- The loudness gradient of the envelope (a quiet slot dim, a loud one accent) — with the palette change later.
- Meters on other screens; input meters on the settings screen stay as they are.

## How to verify

1. Pads screen: pads with sounds show their envelope over 3 rows (a kick = a spike at the start, a pad = a hill), empty pads show a thin line; switching banks switches the strip.
2. Hit pads harder and louder: the L/R bars rise with the sound in dB; a short mark stays at the peak.
3. Layer enough loud pads to clip: the overload light comes on and stays until the next clean hit; the render check proves the numbers match the sound.
