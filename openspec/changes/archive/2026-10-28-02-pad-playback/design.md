# Design 02

## Logical actions

Input becomes actions: `pad.trigger { pad, velocity }`, `pad.release { pad }`, `bank.next`, `bank.prev`, `pad.mode_next`, `pad.volume { delta }` and so on. The screen and the engine know only actions. The `input` crate is pure logic: key event → action, tested without a terminal.

## Keyboard

`config/keymap.toml`. Key release comes through the kitty extended protocol. In a terminal without the protocol there is a simplified mode: no release (gate sounds like one-shot) and a warning.

Ghostty sends key auto-repeat as plain presses, not as repeat events (observed with a probe: 31 presses, then one release). A pad press is therefore ignored while that pad key is held.

Ghostty also sends the base-layout key (`1081:1049:113` for Russian Й on the physical Q key), but crossterm 0.29 drops it. Pads are mapped with a US + Russian table; other layouts would need our own parser of the kitty key sequences (decided not to do it for now).

## Voices

32 sounding voices plus a reserve for fading ones: a cut voice (a repeated hit, release in gate, not enough voices) fades out over a few milliseconds instead of stopping abruptly. Pad volume is smoothed inside the block.

## Test sounds

Pad `i` of bank `b` is a decaying tone `i + 3·b` semitones above 110 Hz, about one second long: you can hear different pads, a bank change, and the release in gate.
