## 1. Config
- [x] 1.1 Shared user config on top of `config/audio.toml`; writing through a temporary file; a broken file is not overwritten

## 2. Sound
- [x] 2.1 Input ports, input level peak in an atomic
- [x] 2.2 Output and input device selection by name; connection recalculation by difference; fallback device if the selected one is missing
- [x] 2.3 Reconnection after JACK port registration
- [x] 2.4 Buffer change on the fly
- [x] 2.5 Tests for device selection and connection difference

## 3. Screen
- [x] 3.1 F12 — settings screen: output, input, buffer, input meter; help at the bottom
- [x] 3.2 Live check: a second JACK client instance does not break the connections of the first
