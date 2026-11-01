# Design 03a

## Samples into the audio thread

A sample is decoded, resampled and wrapped in an `Arc` on a loader thread. The UI sends it to the audio thread in a command. A voice holds its own `Arc` clone (an atomic increment, no allocation), so replacing a pad's sample never cuts a playing voice.

The audio thread never frees memory: when a voice ends or a pad's sample is replaced, the old `Arc` is pushed into a garbage queue back to the UI thread, which drops it. If that queue is ever full, the `Arc` is leaked instead of freed in the audio thread (counted in the log).

Loads carry a per-pad generation number: a slow older load that finishes after a newer one for the same pad is ignored.

## Decoding

`symphonia` (MPL-2.0, compatible with GPL-3.0) decodes WAV, FLAC, AIFF, MP3, OGG/Vorbis. `rubato` (MIT) resamples to the system rate. Samples are stored as 32-bit float, mono or stereo; a mono sample plays the same on both sides. Loading progress goes to the status line.

## Assigning "by hit"

On the files screen Enter on a file takes it "in hand"; a pad hit (keyboard or controller) puts it on that pad instead of playing. Without a file in hand, pads play as usual on every screen.

## Session

`~/.local/share/arasaka-404/session.toml` (`$XDG_DATA_HOME`): `version`, the last browser folder, and per pad its file path, mode and volume. Written through a temp file on every change and on exit. Samples are not copied. A file that is gone marks its pad "missing"; the pad stays silent until a new sample is put on it. A newer format will come with a migration keyed by `version`.
