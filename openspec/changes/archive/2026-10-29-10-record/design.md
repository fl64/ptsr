# Design — 10

## Capture path

The JACK process callback already receives the input ports (the level meter reads them). A **recording ring** is allocated once at start: 30 s × rate × 2 channels of f32 (about 11 MB), owned by the audio side, written only from the audio thread:

- `recording: AtomicBool`, `source: AtomicU8` (mic = the in ports, output = the `rec_l`/`rec_r` tap ports connected to the current output device's monitor ports), `frames: AtomicUsize`.
- The app drives it directly through the shared `Arc<Recorder>` (`start`/`stop`), no engine commands needed — the audio thread only polls `recording` and appends frames; past the capacity the recording stops by itself (the flag clears, the app sees the length at the cap and reports it).
- No allocation, no locks in the callback: the ring is preallocated, the app reads it only after the recording stopped.
- The monitor tap records **everything the output device plays** — ptsr and other programs alike — which is what "record the output" means for sampling. Routing stays on the system defaults; the tap connects to `<device>:monitor_*` at record start and reports honestly when the device has none. On stop the tap is unlinked again, so the patchbay shows ptsr tied to the monitor only while it records.

## From ring to pad

On stop the app thread:

1. Builds a `Sample` from the ring (stereo; mono when one channel is silent throughout — kept as is, the engine plays it fine).
   The first and last 5 ms fade in and out (the engine's cut fade length): a recording starts wherever the key was pressed, often in the middle of a sound.
2. Writes a 16-bit PCM WAV to the samples root: `rec-YYYYMMDD-HHMMSS.wav` (a small writer in `app/src/wav.rs`; symmetric with the decoder, round-trip tested).
3. Puts the file path in hand (the browser's existing `in_hand`), status "recorded · hit a pad". A pad hit then runs the usual put flow — including the overwrite question and the "missing file" marks if the file is later removed.
4. Silence check: peak below −60 dB → the file is not written, the status says the recording was silent.

The hold semantics: Ctrl+R maps to a "hold" action — the keymap resolves its **release** to the stop action (the kitty protocol provides releases; without it, a second press stops too — same fallback as gate pads).

## Keys and actions

- `Ctrl+R` = `record.mic` (hold: press starts, release stops), `Ctrl+T` = `record.output` (toggle). Both free today; R/T on their own stay pad/other keys. Both are GLOBAL actions, so MIDI learn can bind buttons — a controller button toggles.
- Recording works on every screen; the pad hit afterwards works on the pads and files screens as the file-in-hand flow already does.

## Open questions

1. **Ctrl+T for the output.** The owner suggested Shift+R — Shift on the pad block is full velocity, so a different key is needed; Ctrl+T taken unless the owner renames it.
2. **30 s cap.** Enough for a bar or two of ideas; keeps the ring at ~11 MB.
3. **The output tap stays linked after a stop.** Decided: unlink on stop — the owner asked for the tidy behavior.
4. **The first moments of an output recording.** PipeWire may establish the monitor link a few milliseconds after the start, so the head can be silent; not measured. Default: leave as is, the render and the owner's listening decide.
