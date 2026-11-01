# Design 02b

## Device profiles

`config/devices/<name>.toml`: a `controls` section (hardware: CC, notes, absolute/relative type) and `screens.<screen>` sections (control → action). A device is recognized by part of its MIDI port name. A profile without a screen section = pads and banks only.

## Threads

- `midi-scan` polls the MIDI ports every second (midir reports no hotplug), connects new ports and drops gone ones. Up to one second passes before a plugged-in controller works.
- `midi` resolves messages with the device profile. Pad hits go straight into the audio thread through their own queue (rtrb is single-producer, so the audio thread reads two queues: UI and MIDI); waiting for the UI frame would add up to 33 ms. Everything else (buttons, knobs, debug log) goes to the UI thread, where the selected pad and takeover state live.
- The current bank and screen are shared with the MIDI thread through atomics. A release goes to the pad that was pressed, even if the bank changed. When a device disappears, its held pads are released, so a gate pad does not hang.
- A profile file with an error is reported for 5 s with its file and line and skipped; its device falls back to the default profile.

## Soft takeover

For each absolute control the program stores "picked up / not picked up". After the target changes, the control is not picked up until its value crosses the current value of the parameter. The screen shows which way to turn.
