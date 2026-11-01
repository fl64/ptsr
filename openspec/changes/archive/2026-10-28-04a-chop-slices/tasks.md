## 1. Engine
- [x] 1.1 A pad plays a region `start..end` of its sample; a region change does not affect a playing voice; offline render test
- [x] 1.2 Preview of a region with the playback position readable from the UI thread

## 2. Chop screen
- [x] 2.1 F3 opens the chop screen for the selected pad's sample; Esc goes back; empty pad gives a status message
- [x] 2.2 Braille waveform, markers, selected slice, playback cursor
- [x] 2.3 Tab: equal parts 2/4/8/16/32
- [x] 2.4 Space play/stop, M cut at the cursor
- [x] 2.5 Select, nudge (coarse/fine), delete markers; Ctrl+Z undo
- [x] 2.6 Pad keys play slices on the chop screen

## 3. Slices to pads
- [x] 3.1 Enter lays slices out from the selected pad across banks; status message; overflow after bank 8
- [x] 3.2 Session v2 with regions; v1 migration; test on an old file
- [x] 3.3 Chop actions in the action list and MIDI learn (group CHOP)

## 4. Verification
- [x] 4.1 `renders/04-chopped-beat.wav`
- [x] 4.2 Docs: README and `docs/` updated for the chop screen

## 5. Owner feedback: slices and modes
- [x] 5.1 Selection by slice; nudge and delete act on the slice start; the first slice trims the sample start; undo keeps cuts and range
- [x] 5.2 Knobs `slice.start` / `slice.end` with soft takeover; in MIDI learn
- [x] 5.3 Assign mode: real pads of the bank, plan in order / by hand, bank change, summary, apply, Esc, plan undo, pad hit plays the planned slice
- [x] 5.4 Pads named after their slice ("break 5"), saved in the session
- [x] 5.5 Docs: README and `docs/ui/chop.md`
