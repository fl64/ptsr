# Set up the SMC-PAD

1. Connect the device over USB.
2. Select device preset 5 with SHIFT and the printed PAD5.
3. Open F1 and select Controller. On SMC-PAD, check for `encoders ready`.
4. Open F12 MIDI. The verified USB device offers `SMC-PAD PAD BANK`.
5. With `Open panel`, press PAD BANK once to open the panel. Press it again
   to return to the playing grid. The device keeps its selected layer after release.
   With `Off`, the first pad hit on each layer updates the panel.

If a saved map has old assignments, select `SMC-PAD layout review` in F12 MIDI.
Compare the current and proposed roles. Accept the roles to replace.

The device Note Repeat uses its own clock. Leave it off when you use ptsr.
The supported repeat path is ptsr Note Repeat in change 100.

SHIFT with the printed PAD13–16 changes transpose or octave. If ptsr reports
shifted pads, reset transpose and octave on the device. If it reports another
preset, select preset 5. See the [hardware map](plans/SMC-PAD-INTEGRATION.md#1-what-the-smc-pad-gives-the-computer).

See [SMC-PAD paths](CONTROLLER-PATHS.md) for button and pad routes.

## Check the layout

Use a disposable project with a break on pad 1 and a different sound on pad 2.
On the panel, the P numbers are the numbers printed on the device. Each cell shows its number.

1. On Pads, use LEFT and RIGHT to change banks. Turn bank 1 knob 1.
   Check that the selected pad stays the same.
2. Open Chop. Turn knob 2 slowly in both directions. Turn knobs 3–4 for the
   selected slice. Turn knobs 5–6 for the whole Chop region.
3. On the visible panel, press LEFT for the screens view, then P12 Help. Check that F1 shows the
   controller. Select knobs 3–6 on the diagram to read their full actions.
4. Return to Pads. Turn bank 1 knobs 2–7 and bank 2 knobs 1–7. Check that no pad
   value changes. Open pad edit. Press PAD BANK to hide the panel, then RIGHT to change the page. On TONE, turn
   knobs 7, 5 and 3. Check volume, pitch and mute group, from top to bottom.
   Check that the knob strip marks the knob and shows the value.
5. Start playback. Turn knob 8 in each bank. Listen for smooth
   changes in volume. Press STOP. Check that playback pauses and the panel stays as it was.
6. In the screens view, press P16 A1. Hit P6 Tap in time. Check the BPM.
7. Start Copy from Pads. Select an empty target in the 12 upper cells on the same layer.
   Copy to an occupied target. Use P11 to cancel, then try again and use P12.
   Play the source and target. Check the two sounds.
8. Start Assign in Chop. Choose a target on the same layer. Press P4 Apply.
   Repeat with a target on P13–P16 of the playing layer. Return to Apply.
9. Press PAD BANK twice with Open panel. Check that the playing grid returns
   before a pad hit. Repeat a screen change with Open panel, then with Off. Count PAD BANK,
   reveal hits and action presses. Use the [target counts](CONTROLLER-PATHS.md#target-routes)
   for Copy and Assign. Record the route and the result.

See [change 126 checks](../openspec/changes/126-knob-layout/proposal.md)
for the full knob and keyboard pass. Listen to the transport reference at
`renders/92-command-panel.wav`.
