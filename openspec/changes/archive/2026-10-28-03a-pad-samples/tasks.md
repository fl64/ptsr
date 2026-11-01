## 1. Engine
- [x] 1.1 Stereo samples per pad as `Arc`; voices hold their own reference; replacing a pad sample keeps playing voices
- [x] 1.2 Garbage queue back to the UI: the audio thread never frees memory; no-allocation test still passes

## 2. Loading
- [x] 2.1 Decoding WAV/FLAC/AIFF/MP3/OGG on a loader thread, resampling to the system rate; tests on generated files
- [x] 2.2 Progress and errors in the status; stale loads ignored by generation

## 3. Files screen
- [x] 3.1 F6 files screen: folders and audio files, navigation, starts in ~/Music or home, then the last folder
- [x] 3.2 "File in hand": Enter takes a file, the next pad hit puts it on the pad, Esc cancels

## 4. Session
- [x] 4.1 `session.toml`: save on change and exit, restore on start (samples, modes, volumes, folder)
- [x] 4.2 A missing or broken file marks the pad and does not crash the program
- [x] 4.3 `renders/03-pads-from-files.wav`

## 5. Files screen layout
- [x] 5.1 Pad grid of the current bank left of the file list
- [x] 5.2 File in hand highlighted: the file line, the grid, the help line
