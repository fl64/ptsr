# Design 38

## Decisions (owner, 2026-10-31)

- 1.1 color by frequency; 2.1 only the chop wave of F3; 3.1 three bands blended in the
  palette of the program (warm, light, cyan) — replaced after the first look.
- The first look (owner): "the color barely reads and merges". Measured on the test
  breaks: the band shares average 0.37 / 0.31 / 0.32, a near-even mix is grey. Now
  ("something contrasting", 1.1): red, green, blue; each band against its own average
  over the file; the shares to the 4th power, so the band that stands out takes the
  column; the other slices at 0.8 brightness instead of 0.55.
- The second look (owner): "the dimmed colors do not show which slice is current". The
  selection leaves the brightness: the selected slice sits on a lighter background in the
  wave rows and in the numbers row (a text row, so it shows with the kitty picture too);
  every slice inside the region is drawn at full color, outside it dim.
- The third look (owner, a screenshot): the grey plate "looks more like inactive". The
  plate takes a dark tint of the selection cyan (#0B333D): selected, not disabled.

## Model

- The bands are measured once when a chop is made: a one-pole low-pass (~200 Hz) gives
  the low, a one-pole high-pass (~4 kHz) the high, the rest is the mid; the RMS of each
  per block of 256 frames. No dependency; a pass over the sound, on the UI thread when
  the chop opens. Ceiling: a very long file delays the first open by one pass; move it to
  the loader thread if that shows.
- The color of a column: the band weights, normalized, mix the three colors; the
  selection scales the brightness (selected 1.0, other slices dimmer, outside the region
  dimmest).

## Open questions

None.
