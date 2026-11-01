# 03b — Browser comfort

## Why

With many samples, finding the right one needs a filter, favorites and hearing files while scrolling.

## What changes

- Filter by name: `/` opens a text line.
- Favorite folders (Ctrl+B).
- Preview: the selected file plays while scrolling; Space toggles it.
- Duration and a mini waveform of the selected file.
- Swapping two pads: Ctrl+X takes the selected pad in hand, the next pad hit swaps them; Esc cancels.

## Out of scope

A shared library with hashes and tags.

## How to verify

F6, `/`, type "amen": the list narrows. Scroll: each file sounds, the previous one stops without a click. Ctrl+B on a folder: it appears at the top of the list after a restart.
