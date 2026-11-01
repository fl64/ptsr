# Design 03b

## Preview

The preview is a separate engine voice, not a pad: pads keep playing and can be hit meanwhile. A new preview fades the old one out over 5 ms. The file is loaded by the same loader thread as pad samples, in a slot of its own after the 128 pads, with its own generation number, so a late reply for a file the selection already left is ignored.

The loader takes all requests waiting in its queue before each load and keeps only the newest one per pad and per preview, so scrolling fast does not decode every file it passed. A load already running is not interrupted.

The preview plays only while the files screen is open.

## Filter

The filter keeps the folder listing and shows the entries whose name contains the text, case aside, folders included. Changing the folder clears it. While the line is open, key presses and repeats type text and never reach the keymap, so pads stay silent; key releases and Ctrl+key still pass, so a held gate pad is released and Ctrl+Q quits.

## Favorites

Ctrl+B on a folder adds it to or removes it from the favorites. Favorites are listed first, marked ★, in every folder except themselves; Enter on one goes straight there. They are saved as `[browser] favorites` in the user config `~/.config/arasaka-404/config.toml`, next to `[audio]` and `[midi]`.

## Duration and mini waveform

They come from the sample the preview already loaded: 48 blocks, each the peak of its stretch, scaled to the loudest one. With the preview off, the file is not loaded and neither is shown.

## Swapping pads

Ctrl+X takes the selected pad in hand (shown with a magenta frame); the next pad hit, from the keyboard or a controller, swaps the two: sample, mode, volume and "missing" mark. The engine swaps the pads and moves their playing voices to the new index, so a voice keeps its own volume and fades out without a click. A swap is refused while either pad is loading, since the load would land on the old place. While a pad is in hand, controller hits do not sound.
