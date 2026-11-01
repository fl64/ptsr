# Design 07

The engine applies the master gain to the mixed output, ramping from the old to the new value within a block, like pad volumes. The UI keeps the value in dB, sends it as a linear gain, and stores it in `session.toml` (`master_db`, default 0; older sessions without it read as 0, no migration needed).

The knob action goes through the same soft takeover as the pad volume, with a fixed target, so it never jumps.
