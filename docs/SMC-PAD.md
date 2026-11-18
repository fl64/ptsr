# Set up the SMC-PAD

1. Connect the device over USB.
2. Select device preset 5 with SHIFT and the printed PAD5.
3. Open F1 and select Controller. On SMC-PAD, check for `encoders ready`.
4. Open F12 MIDI. The verified USB device offers `SMC-PAD PAD BANK`.
5. Use `Open panel` to open the command panel on each detected PAD BANK press.
   Use `Off` to open it with the first hit on the command layer.

If a saved map has old assignments, select `SMC-PAD layout review` in F12 MIDI.
Compare the current and proposed roles. Accept the roles to replace.

The device Note Repeat uses its own clock. Leave it off when you use ptsr.
The supported repeat path is ptsr Note Repeat in change 100.

SHIFT with the printed PAD13–16 changes transpose or octave. If ptsr reports
shifted pads, reset transpose and octave on the device. If it reports another
preset, select preset 5. See the [hardware map](plans/SMC-PAD-INTEGRATION.md#1-what-the-smc-pad-gives-the-computer).

See [SMC-PAD paths](CONTROLLER-PATHS.md) for button and pad routes.
