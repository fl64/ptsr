# 16 — The live strip gets a fourth row

## Why

At three Braille rows (12 dot rows) the live output strip shows either a thin line or a fat block: the owner says quiet and loud merge into almost the same blob. A fourth row adds four more dot rows of height and the picture separates.

## What changes

- The live output strip on the pads screen is **4 rows tall** instead of 3.
- The meters keep their three lines (L, R, CLIP) at the top of the same column; the fourth row under them stays empty as a gap before the pad info.

## How to verify

1. Pads screen: silence is a thin line in the middle of four rows; a quiet pad draws a low wave, a loud one reaches the top — visibly different, not two kinds of the same blob.
2. The pad grid still fits: header + 4 strip rows + 16 grid rows on a 24-line terminal.
