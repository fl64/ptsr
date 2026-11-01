# Design 05

## Device

A device is the prefix of a JACK port name up to the last `:` (`HUAWEI FreeBuds 7i:playback_FL` → `HUAWEI FreeBuds 7i`). The config stores this name. Device ports go in JACK order; a mono device gets both channels.

## Connection

The program owns the connections of its ports: on each recalculation it keeps exactly the needed connections. A recalculation is the difference between the current and the needed connections: extra ones are removed, missing ones are added, matching ones are not touched, so events from other programs do not break the sound. Manual connections from qpwgraph for arasaka-404 ports are overwritten on the next recalculation.

A recalculation runs: at start, on a selection in the settings, after a JACK port registration event. The JACK callback only raises a flag, the recalculation is done by the UI thread: calling JACK from the notification callback is not allowed.

## Config

`config/audio.toml` is read (defaults, from the launch folder), with `~/.config/arasaka-404/config.toml` (`$XDG_CONFIG_HOME`) on top of it. Writing goes only to the user file, through a temporary file and a rename. If the user file cannot be read, the program does not overwrite it and reports this.

## Input level

The audio thread computes the input peak per block and writes the maximum to an atomic; the UI takes it and resets it. The input does not go into the engine.
