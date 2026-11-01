# 10 — Record into a pad

## Why

A sampler should also be a recorder: capture an idea from the microphone or the program's own output and have it on a pad immediately, without leaving the program and without touching a file manager.

## What Changes

- **Hold Ctrl+R to record the microphone**: pressing starts, releasing stops. While recording, the status counts the seconds and the corner dot blinks.
- **Ctrl+T records what plays on the output device right now** — ptsr and other programs alike: the tap sits on the monitor ports of the current output. Press to start, press again to stop.
- **After stopping, the recording is "in hand"**: the status says "hit a pad", and a pad hit binds it — with the usual question when the pad already holds something. Esc drops the recording.
- Recordings are kept as **WAV files in the samples root** (`~/ptsr/samples` by default), named by time (`rec-20261028-1845.wav`); pads bound to them survive a restart.
- Routing stays on the system defaults for now (input and output); the fine device tuning is a later change.
- Recording stops by itself at **30 seconds** with a status; the meter of silence is not recorded — a recording with no signal at all is dropped with a message.
- Both actions are learnable on a controller (MIDI learn, GLOBAL).

## Capabilities

New: `recording`. Modified: none — binding a recording uses the existing "file in hand" flow.

## Out of scope

- Fine device choice for recording (the system defaults are used, as everywhere today).
- Recording into the chop screen, overdubs, punch-ins.
- Project folders as the destination (lands with the projects change).

## How to verify

1. Hold Ctrl+R, speak, release: the status says the length, "hit a pad"; hit pad Q — the recording plays from it; the question appears when Q is taken.
2. Ctrl+T while a pad loops: stop — the captured loop is in hand; bind it and listen that it matches what played.
3. `~/ptsr/samples` holds `rec-…wav` files; restart — the bound pads still play them.
4. Hold Ctrl+R for half a minute: it stops by itself with a message.
5. Ctrl+R with no microphone connected: a clear status, the program keeps working.
6. F12 MIDI: "record microphone" and "record output" in GLOBAL; learn a button for each.

Listen for: the recorded bound pad does not click at the start; the output recording matches what was heard.
