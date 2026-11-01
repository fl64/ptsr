# Design 06

The UI keeps a "held" flag per pad. Keyboard: set on press, cleared on release (the release goes to the pad that was pressed, as for audio). Without the kitty protocol there are no releases, so the keyboard does not set the flag and pads only blink. Controller: the MIDI thread sends a release event to the UI next to the one it sends to the audio thread, including releases it makes when a device disappears.

A pad is drawn lit when it is held or was hit less than 120 ms ago.
