# 78 — The mode glyphs the other way round

## Why

The owner (2026-11-02) reads the glyphs of 77 the other way: "`⇢` I read as: pressed and
off it goes; `⇥` as: I hold until it stops". So one-shot is `⇢` and gate is `⇥`.

## What changes

- The mode line of a pad shows `⇢` for one-shot and `⇥` for gate: `⇢ +1 mute 2`.
- F1 names `⇢` one-shot and `⇥` gate.

## Out of scope

- Any other glyph or text on the pad.

## How to verify

1. Load a sample on a fresh pad: the line under its name is `⇢`.
2. Press `H`: the line is `⇥`.
3. Press F1: it names `⇢` one-shot and `⇥` gate.
