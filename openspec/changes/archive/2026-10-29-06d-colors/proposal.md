# 06d — Bank colors

## Why

Eight banks look alike: the owner wants to tell at a glance which bank is open and which bank a pad belongs to. Asked for: each bank its own color, soft tones but clearly different, and the pads in the color of their bank.

## What changes

- Every bank has its own muted color: coral, amber, olive, green, teal, blue, violet, orchid for banks 1–8.
- The bank row shows each bank's number and fill glyphs in its color; the open bank stays bold in brackets.
- The pads of the open bank have their frame and key letter in the bank's color, on every screen with the pad grid.
- The meanings stay: a selected pad is cyan, a pad in hand magenta, a hit flashes white; the assign mode of the chop screen keeps its own bank marks (planned magenta, taken bright, empty dim).

## Out of scope

- Choosing colors in the config (`[colors]` with migration).
- The envelope gradient by loudness (13-session-overview).

## How to verify

1. Pads screen: switch banks with PgUp/PgDn: the pad frames change color with the bank; the bank row shows all eight colors, the open one bold in brackets.
2. Select a pad: its frame is cyan; hit a pad: white flash; F6 with a file in hand: magenta as before.
3. F6 and F3: the pad grid there has the same bank color.
