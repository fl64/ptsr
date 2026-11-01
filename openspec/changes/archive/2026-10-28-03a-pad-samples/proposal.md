# 03a — Your own samples on pads

## Why

To play your own sounds, you need to find a file, put it on a pad and play it — without a mouse — and find everything in place after a restart.

## What changes

- A files screen (F6): the pad grid of the current bank on the left, folders and audio files on the right; arrows to move, → or Enter to open a folder, ← to go up; banks switch here too. It opens in `~/Music` (or the home folder) the first time, then in the last folder.
- "File in hand": Enter on a file takes it; the next pad hit puts it on that pad. Esc puts it back.
- Formats WAV, FLAC, AIFF, MP3, OGG, decoded in the background and resampled to the system rate; the sound never stops while a file loads.
- Pads play samples in stereo. Empty pads keep the test tone.
- The session — samples, modes and volumes of all 128 pads, the last folder — is saved on every change and restored on start. A missing file marks its pad "missing", the others work.

## Out of scope

Filter by name, favorite folders, preview while scrolling, mini waveform, swapping pads (all in 03b); a shared library with hashes and tags.

## How to verify

1. `task run`, F6. Find a break, Enter, hit W on the keyboard: pad W plays the break.
2. Load a 5-minute file while pads play: no dropouts, the status shows loading.
3. Close and open the program: the break is still on W.
4. Rename the file on disk and start again: W shows "missing", other pads work.
5. Listen to `renders/03-pads-from-files.wav`.
