# Show the samples that sound on Pads

## Why

The owner can hear several slices at once. The Pads screen does not show where each slice plays in its sample.

## What Changes

- Show the full sample wave with pad ranges, highlighted sounding ranges, and playback lines.
- Use compact rows for other sounding samples. Name the pads for samples that do not fit.
- Keep the wave visible in silence. Follow the last sounding pad or pin the selected sample.
- Switch between Sound and Rhythm from the keyboard or SMC-PAD.
- Keep live sample indicators in the strip. Keep the grid available for pad selection and edit feedback.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `screen-layout`: Pads source and rhythm views.
- `controller-reach`: keyboard and SMC-PAD routes for both actions.
- `command-panel`: the states of the strip and pin cells.

## Impact

The audio thread supplies bounded read-only voice data. The app groups this data by sample. The terminal draws the result within the current strip.

## How to check

### Keyboard

1. Place slices of one sample on two pads. Place a second sample on a third pad.
2. Play the three pads. Check two lines on the main wave and a row for the other sample.
3. Let the samples end. Check that the main wave stays and says `silent`.
4. Press Alt+9 twice. Check Rhythm, then Sound. Select a loaded pad and press Alt+0. Check that the wave stays on this sample as other pads play.
5. Repeat at 80x26 and 120x40. Play more samples than the strip can show. Check the pad names next to the extra sample count.

### Controller

1. Play the same slices from SMC-PAD.
2. Hold PAD BANK and press P16. Release PAD BANK. Check Rhythm. Repeat to show Sound.
3. Open Edit from the Pads panel. Hold PAD BANK and press P16 in Pad Edit. Release PAD BANK. Leave Edit. Check that another pad hit keeps the pinned sample.

### Views and sound

Compare `renders/screens/pads-source-live-80x26.ansi` and `renders/screens/pads-source-live-120x40.ansi`. Check `renders/screens/pads-source-overflow-80x26.ansi` and `renders/screens/pads-source-overflow-120x40.ansi` for the hidden pad labels. Listen to `renders/139-source-view.wav` while checking the playback lines in a matching project.
