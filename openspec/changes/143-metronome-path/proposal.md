## Why

The click helps the owner play in time. It should stay in the headphones while the owner records the output.

## What Changes

- Give the click, including the A1 count-in, a separate monitor path.
- Keep the click out of the master meter and the recorded mix.
- In JACK, retain the other applications on the selected output in Ctrl+T recordings.
- Give the click a saved 0 to 100 percent level in Settings Audio. A controller knob can change it.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- transport: click level and separation from the master.
- recording: record playback before the click.
- screen-help: show the level and name the recording path.

## How to check

### Keyboard

1. Open F12 Audio. Set Click level to 50 percent.
2. Start the click on Pads. Check that it sounds without moving the master meter.
3. Start a second application on the same output. Start Ctrl+T and play a pad. Stop the recording and play it back.

### SMC-PAD

1. Hold PAD BANK and press LEFT, then P11 Setup. Release PAD BANK. Select Audio.
2. Turn E1 to Click level. Turn E2 to change it. Start the click with a PLAY hold.
3. On Pads, start output recording with P14. Play a pad and use STOP. Play the take.

### Screens

Check `renders/screens/settings-normal-80x26.ansi` and
`renders/screens/settings-normal-120x40.ansi` for the click level.

### Sound

Compare `renders/143-metronome-monitor.wav` and `renders/143-metronome-record.wav`.
Both have the pad and the outside source. Only the monitor has the click.
