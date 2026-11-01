# 02 — Pads from the keyboard

## Why

Playing pads is the core of the instrument. The keyboard comes first: it is always there, and you can try it without hardware. MIDI controllers come in a separate change, 02b.

## What changes

- A grid of 16 pads and banks 1–8 (128 pads).
- Pads from the keyboard: a 4×4 block on physical keys, with key release, also with the Russian layout.
- Pad modes: one-shot and gate; volume follows velocity; pad volume with the `−` `=` keys.
- 32-voice polyphony, a voice is cut without a click.
- The "Pads" screen (F2) after the sketch `docs/ui/pads.md`, help at the bottom, full help on F1.
- Key layout from `config/keymap.toml`.

For now the pads play built-in test sounds: each pad has its own note, banks shift the pitch. Samples from disk come in 03.

## Out of scope

MIDI and controller profiles (02b), layers, mute groups, keygroup programs.

## How to verify

1. `task run`, F2. Hit `1 2 3 4 / Q W E R / A S D F / Z X C V` — each pad has its own note, the pad flashes.
2. Switch the layout to Russian — the pads work the same.
3. Shift + pad — louder.
4. Select a pad with the arrows, Enter — gate mode; hold the key — the sound holds, release it — it fades out without a click.
5. PgDn — bank 2, higher notes; pads of bank 1 that are sounding play to the end. Ctrl+3 — straight to bank 3.
6. Listen to `renders/02-pads-velocity.wav`.
