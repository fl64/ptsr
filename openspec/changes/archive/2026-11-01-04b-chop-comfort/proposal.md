# 04b — Chopping comfort

## Why

04a gives a working chop, 04c keeps it linked to the pads, 04d finds the hits. What is left is shaping the sound of a pad and seeing it well. After 19 the sound settings live on the pad: 04b adds the envelope on the same model.

## What changes

- **Pad envelope**: attack at the start of the piece, decay towards the end of the piece (the tail fades to silence right at the end), release in gate mode (after the release the sound fades over it). Edited on the pads screen: `Tab` steps through volume → pitch → attack → decay → release, `−` `=` change the picked one, the change scope (`Ctrl+A`, marks) works as in 19. The value pops up when it changes.
- **A large view of the selected pad's piece** on the pads screen, with its envelope drawn on it.
- **The waveform in kitty graphics** where the terminal supports it (Ghostty does); Braille stays as the fallback.
- ~~**The chop screen profile for the X-Session Pro**~~ — moved to 21-edit-mode: the profile follows the parameter standard there (owner decision, 2026-10-30).

## Out of scope

Reverse (a separate small change can add it later), full ADSR (sustain, hold), auto-chop by hits, snapping, tempo grid (04d). Saving slices to new files, resampling, time-stretch.

## How to verify

Take a break chopped into slices, lay the slices out on pads. Set an attack on one pad: the start comes in softly, no click. Set a decay on another: the tail dies right at the end of the piece, at the same place as before. Hold a gate pad and release it: the release fades without a click. Switch the scope to the bank and change the release: every pad of the bank follows. Watch the big view: the envelope moves when you change a value. Then play your own beat: first with the keyboard only, then with the X-Session Pro only.
