# 05 — Audio settings: input, output, buffer

## Why

Now the sound goes to the first device found, and you can change it only in qpwgraph. We need to choose the output and the input right in the program, from the keyboard, and keep the choice saved.

## What changes

- A settings screen on F12: output device, input device, buffer size; ↑ ↓ — row, ← → — value, applied at once.
- The choice is applied at once, without a restart and without a break in the sound.
- An input level meter: you can see that the signal comes in.
- A shared user config `~/.config/arasaka-404/config.toml`; the program writes the choice there itself. `config/audio.toml` in the repository holds the defaults.
- The device is gone — the sound goes to the first available one; it is back — the sound returns to it by itself. The saved choice is not lost.
- Input debug from 02 will become a tab of this screen (in 02).

## Out of scope

Recording from the input, listening to the input (monitor), device choice in the `cpal` fallback output mode, MIDI settings, moving `keymap.toml` into the shared config.

## How to verify

1. `task run`, F12. With the → arrow select the laptop speakers, press Space: a click from the speakers. Select the headphones → a click in the headphones.
2. Select the microphone as input, say something: the level meter moves.
3. Buffer 256 → the status line shows 5.3 ms, clicks without crackle.
4. Turn off the headphones while they are selected: the sound moves to the speakers, a message at the bottom. Turn them on — the sound is back in the headphones.
5. Close and open the program — the choice is in place.
