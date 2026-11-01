# 26 — The microphone is linked only when it is needed

## Why

The system sound broke twice (2026-10-29 21:33, 2026-10-30 09:01): the Bluetooth
headphones stopped getting cycles until they were reconnected or the audio services
restarted. The second time it broke in the same second the program started. The program
links the USB microphone at every start, recording or not; the microphone has the higher
driver priority (2009 against 1010), so at every start and quit the graph moves the
headphones from their own clock to the microphone's and back (seen live 2026-10-30
10:08). The suspect is this handover. Not proven: the change removes it from the start
and the quit and leaves it only where the microphone is really used.

## What changes

- **The microphone is linked only while it is used**: while a microphone recording runs
  and while the settings screen (F12) is open, so its input meter still moves there. At
  start and quit the program does not touch the microphone. (Owner: "go ahead", option
  "only while recording", 2026-10-30.)
- The input device choice and its name in the settings stay as they are; the config does
  not change.

## Out of scope

The output recording tap (it is linked only while recording already); the input meter on
other screens; a setting to keep the microphone always linked (offered, not chosen).

## How to verify

Live: music in a browser, start the program: `pw-top` shows the headphones driving, no
microphone in the graph. Hold `Ctrl+R` and speak: the recording has the voice; release:
the microphone leaves the graph. F12: speak, the input meter moves; leave F12: the
microphone leaves. Over the next days: does the sound still break at a start?
Tests: the routing asks for no input link at start, one while recording and on F12,
none after; recorded-event tests for `Ctrl+R` and the F12 entry and exit.
