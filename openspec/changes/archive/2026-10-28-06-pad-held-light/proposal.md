# 06 — Pad stays lit while held

## Why

The pads screen only flashes a pad for a moment on a hit. When a pad is held (gate mode, a long sound), the screen does not show it, so you cannot see which pads are down.

## What changes

- A pad stays lit for as long as it is held — from the keyboard and from a controller.
- A short tap still lights the pad for at least a moment, so it is visible.
- When a controller is pulled out, its held pads go dark.

## Out of scope

Showing that a pad is still sounding after release (one-shot tail), level meters per pad.

## How to verify

`task run`, F2. Hold Q: pad Q stays lit until you let go. Tap W quickly: it blinks. With a controller pad: the same.
In a terminal without the kitty protocol there are no key releases, so keyboard pads only blink.
