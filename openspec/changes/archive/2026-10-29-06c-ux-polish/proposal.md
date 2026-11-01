# 06c — UX polish: bank fill, honest messages, small text fixes

## Why

A hands-on walkthrough of the whole path (start → files → pads → chop → settings) left small but real frictions: the first-steps hint never shows because other messages take its place, audio errors speak in developer words, the chop screen shows bare numbers, and the bank row does not say which bank holds what. The owner also asked to see how full each bank is.

## What Changes

- **Bank fill in the bank row.** Every bank shows two braille glyphs — a 4×4 dot grid that mirrors the pads: a lit dot is a pad with a sound. Works on every screen that shows the bank row, updates at once.
- **The first-steps hint keeps its place.** Short messages (a MIDI controller connects, audio is unavailable) no longer hide "Pads are empty: F6 opens files…" for good; the hint comes back when the message is gone.
- **Audio trouble in plain words.** "No audio" stays as the small indicator in the corner; the status line gets one short plain message ("No sound: the audio server is not available"), internal error codes go to the log only.
- **The files screen names both paths from a file.** With a file selected it says: to a pad (Enter) or straight to chopping (F3). Chopping from the files screen already works — it was just invisible.
- **Short folder in the title.** The files screen shows `~/Music/breaks` instead of the full `/home/…` path.
- **Envelope preview in three rows.** The preview strip of the selected file on the files screen is three rows tall, so quiet and loud parts are clearly readable.
- **Replacing a pad with the same sound asks nothing.** Putting the very file the pad already plays back on it is silent; the scary "cannot be undone" question appears only when something really changes.
- **Chop screen text.** "1 slice, 2 slices" in plain English; the slice line labels its numbers: `slice 1 · start 0.000 s · length 326 ms`.
- **Honest settings message.** With JACK already set in the config but unavailable, the Audio tab says "JACK is not available right now" instead of telling the owner to set what is already set.
- **Screen keys in order.** The screens group of the help line lists F1, F3, F6, F12 in ascending order, not mixed.

## Capabilities

Modified: `pad-playback` (bank row fill), `screen-help` (hint keeps its place), `audio-engine` (plain audio message), `sample-loading` (short folder, both paths named), `chopping` (texts, no same-sound question), `audio-settings` (honest device message).

## Out of scope

- New proportions of the files screen (pad grid vs file list) — a sketch will be offered separately.
- Slice parameters, big slice view, kitty graphics (04b); auto-chop (04d).

## How to verify

By hand, in order:

1. Empty session, controller on: after "MIDI connected" fades, the hint about F6 is on the pads screen.
2. Fill pads 1 and Q in bank 1, one pad in bank 3: the bank row shows two lit dots in banks 1 and 3's glyphs, empty banks stay dark; switching banks keeps it true on F2, F6 and the chop screen.
3. F6: the title reads `~/Music/breaks`; the preview of a quiet-then-loud file shows the difference in its three rows; with a file selected the screen names both Enter (to a pad) and F3 (to chopping); F3 opens the chop screen.
4. Put a file on the pad that already plays it: no question, nothing flashes.
5. On the chop screen with one slice: "1 slice"; the slice line says start and length.
6. Stop PipeWire-JACK and start the program: one plain message, the corner shows "no audio", the log holds the details.
7. In F12 Audio (JACK set, server off): the row says JACK is not available right now.

No audio behavior changes; nothing to listen for beyond the usual pads.
