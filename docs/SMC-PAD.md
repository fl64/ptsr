# Set up the SMC-PAD

1. Connect the device over USB.
2. Select device preset 5 with SHIFT and the printed PAD5.
3. Open F1 and select Controller. On SMC-PAD, check for `encoders ready`.
4. Open F12 MIDI. The verified USB device offers `SMC-PAD PAD BANK`.
5. With `Hold panel`, hold PAD BANK to show the panel. Hit a cell during the hold.
   Release PAD BANK to return to the playing grid. Pads play without a hold on
   either device layer. With `Off`, the first pad hit on each layer updates the panel.

If a saved map has old assignments, select `SMC-PAD layout review` in F12 MIDI.
Compare the current and proposed roles. Accept the roles to replace.

The device Note Repeat uses its own clock. Leave it off when you use ptsr.
The supported repeat path is ptsr Note Repeat in change 100.

SHIFT with the printed PAD13–16 changes transpose or octave. If ptsr reports
shifted pads, reset transpose and octave on the device. If it reports another
preset, select preset 5. See the [hardware map](plans/SMC-PAD-INTEGRATION.md#1-what-the-smc-pad-gives-the-computer).

See [SMC-PAD paths](CONTROLLER-PATHS.md) for button and pad routes.

## Presets

The device keeps 8 presets. SHIFT with the printed PAD1–PAD8 selects one.
A preset holds what each pad, knob and button sends.

| Preset | Pads | PAD BANK |
| --- | --- | --- |
| 1 Performance, 3–8 user | Note pads on channel 10, the same in each preset | Shows the second bank only during a hold |
| 2 DAW | Mackie Control (MCP) pads | Latches. ptsr does not support this preset. |

ptsr uses preset 5. It changes only the knob records of preset 5 to relative steps.
The other presets stay as they are. A read of all 8 presets on 2026-11-18 found no
better preset for ptsr. The PAD BANK light shows the second bank of the device.
On preset 5 it is on only during a hold.

## Check the layout

Use a disposable project with a break on pad 1 and a different sound on pad 2.
On the panel, the P numbers are the numbers printed on the device. Each cell shows its number.

1. On Pads, use LEFT and RIGHT to change banks. Turn bank 1 knob 1.
   Check that the selected pad stays the same.
2. Open Chop. Turn knob 2 slowly in both directions. Turn knobs 3–4 for the
   selected slice. Turn knobs 5–6 for the whole Chop region.
3. On the visible panel, press LEFT for the screens view, then P12 Help. LEFT and RIGHT select the Help tabs. Check that F1 shows the
   controller. Select knobs 3–6 on the diagram to read their full actions.
4. Return to Pads. Turn bank 1 knobs 2–7 and bank 2 knobs 1–7. Check that no pad
   value changes. Open pad edit. Release PAD BANK, then press RIGHT to change the page. On TONE, turn
   knobs 7, 5 and 3. Check volume, pitch and mute group, from top to bottom.
   Check that the knob strip marks the knob and shows the value.
5. Start playback. Turn knob 8 in each bank. Listen for smooth
   changes in volume. Press STOP. Check that playback pauses and the panel stays as it was.
6. In the screens view, press P16 A1. Hit P6 Tap in time. Check the BPM.
7. Start Copy from Pads. Release PAD BANK and hit an empty pad.
   Copy to an occupied pad. Use P1 to cancel, then try again and use P4.
   Play the source and target. Check the two sounds.
8. Start Assign in Chop. Release PAD BANK and hit pads 1 and 16.
   Hold PAD BANK and press P4 Apply. Read the pads that the result names.
9. Hold PAD BANK and release it with Hold panel. Check that the playing grid returns
   before a pad hit. Repeat a screen change with Hold panel, then with Off. Count PAD BANK,
   reveal hits and action presses. Check that a hit cell lights for a short time. Use the [target counts](CONTROLLER-PATHS.md#target-routes)
   for Copy and Assign. Record the route and the result.

See [change 126 checks](../openspec/changes/126-knob-layout/proposal.md)
for the full knob and keyboard pass. Listen to the transport reference at
`renders/92-command-panel.wav`.
