# Design 01

## Workspace

- `engine` — the audio engine: voices, mixing, timing; knows nothing about the UI; works with JACK and offline.
- `input` — keyboard and MIDI → logical actions (filled in 02).
- `tui` — screens on `ratatui`.
- `app` — wiring, configs, logs.

## Threads

- Audio thread (JACK callback): takes commands from an `rtrb` queue, mixes voices, writes counters to atomics.
- UI thread: draws 30–60 frames/s, reads a snapshot of the state.
- Background thread: loading and decoding (filled in 03).

## Offline render

The same `engine`, but called in a loop block by block without a sound card. The scenario is TOML or a Rust struct: a list of events with times in samples.

## Sound output

By default (`config/audio.toml`, `backend = "auto"`) — JACK, fallback — `cpal`. Why JACK: on Ubuntu 24.04 with the `pipewire-jack` package, the JACK library from PipeWire comes first, so the program connects to PipeWire without `pw-jack`; PipeWire accepted a buffer of 128 at 48 kHz and reports xruns itself. `cpal` (ALSA) also gave 128 at 48 kHz, but goes through an extra layer and is fine as a fallback.

The program connects both of its outputs to the ports of one device — the first in the JACK list; a mono device gets both channels. This is not always the default device; device selection is change 05 (F12).

Config: `config/audio.toml` in the launch folder, with `~/.config/arasaka-404/config.toml` on top of it (change 05). Log — `~/.local/state/arasaka-404/arasaka-404.log`.

## Known limitations

- The `cpal` fallback output accepts only the f32 format. On plain ALSA without a sound server the device usually gives 16/32-bit integers — then there will be no sound. Not tested live; the fix is conversion to the device format, when it is needed.
- Where `pipewire-jack` does not replace the system `libjack` (not Ubuntu), the program falls back to `cpal`; the workaround is `pw-jack task run`.
