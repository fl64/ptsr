# 24 — The chop screen opens in silence

## Why

The owner (2026-10-30): "when I go to the sample editor, the sound I started on the pads
keeps playing — is that right?" A long one-shot plays over the slice they want to hear,
and nothing stops it.

## What changes

- **Entering the chop screen hushes the pads**: every sounding pad voice fades out
  without a click as the chop screen opens. (Owner: the sound fades by itself on entry,
  no separate "stop all" command.)
- Pad hits on the chop screen keep playing slices (the listen mode); leaving the chop
  screen stops nothing.

## Out of scope

A "stop all" command; hushing on other screens; the files preview (it already stops when
its screen is left).

## How to verify

By ear: start a long one-shot pad, press F3 — the pad fades out at once without a click;
hit a pad on the chop screen — its slice plays. Offline render: a pad voice, then the chop
entry — no NaN, no clipping, silence after the fade, no step at the cut.
