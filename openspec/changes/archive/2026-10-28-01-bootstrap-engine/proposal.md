# 01 — Audio engine skeleton

## Why

Before making pads and chopping, we need an engine that plays sound stably with low latency and can render offline — otherwise the agent cannot check its work, and the owner will hear clicks.

## What changes

- A Rust workspace split into engine, input, UI and application.
- Sound output through JACK (PipeWire-JACK), fallback — `cpal`.
- The simplest playback: one built-in test sound on a key press.
- Offline render of a scenario to WAV and automatic checks of renders.
- A status line in the terminal: latency, audio thread load, buffer underruns (xruns).

## Out of scope

Pads, samples from disk, MIDI, chopping — the next changes.

## How to verify

Start the program in Ghostty (or kitty), press Space — a metronome click without crackle for 10 minutes in a row; the status shows 0 xruns. Listen to `renders/01-click-pattern.wav`.
