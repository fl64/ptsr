# 07 — Master volume

## Why

There is no way to turn the whole sampler down or up from the program: only per pad, or the system mixer.

## What changes

- Master volume of the whole output, −40…+6 dB in 1 dB steps, changed smoothly without clicks.
- Keys `[` quieter, `]` louder (the same physical keys in the Russian layout), on every screen.
- A knob: "master volume (knob)" in the GLOBAL group of the MIDI tab (F12), with soft takeover.
- The value is shown in the status line and kept in the session.

## Out of scope

A limiter on the output; per-bus volumes.

## How to verify

`task run`, play a pad, press `[` a few times: quieter, the status shows "master −3 dB". F12 → MIDI → "master volume (knob)" → Enter → turn a knob: it changes the master volume. Restart: the value is kept.
