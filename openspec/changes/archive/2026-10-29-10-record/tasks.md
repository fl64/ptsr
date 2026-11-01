# Tasks — 10

- [x] 1. Recording ring in the audio side: preallocated stereo buffer, started and stopped by the app through atomics (no engine commands, see design), mic and output-tap sources, self-stop at the cap, atomics for the app; logic tests for the ring state machine; commit.
- [x] 2. WAV writer (`app/src/wav.rs`, 16-bit PCM stereo) with a decode round-trip test; recordings into the samples root named by time; commit.
- [x] 3. Actions and keys: `record.mic` (hold — release stops, second press without releases too) and `record.output` (toggle), Ctrl+R / Ctrl+T, GLOBAL, MIDI-learnable; recorded-event tests; commit.
- [x] 4. App flow: on stop — silence check, 5 ms fade in and out, WAV, in hand, "hit a pad", the output tap unlinked; pad hit binds with the overwrite question; status messages (no input, cap, silent); tests on recorded sequences; commit.
- [x] 5. Render: an automated offline render of a bound recording (a synthetic input captured by the recorder, then played) into `renders/10-record.wav` with the usual checks; commit.
