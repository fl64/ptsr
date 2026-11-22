# Design

## Decision

The owner approved a Sound and Rhythm strip. On 22 November 2026, the owner placed all new playback indicators in the strip. The owner also approved pad labels next to the hidden sample count.

The current rhythm strip supplies the Rhythm view. The current edit and recording strip paths take priority. The source view uses terminal cells at the current UI rate.

The audio callback already publishes pad positions for Chop and the PLAY badge. Extend the same coherent table with each voice's region and envelope level. Use the current project epoch and a fixed number of slots. The engine uses 32 active slots and up to 16 fade slots. The UI reads the table without a retry loop.

Group voices by the identity of their loaded sample. Use the newest sounding voice for automatic focus. Use the most recent voice in each other group to order the compact rows. Use current pad identities to reject old data after a sample replacement. Keep the last valid wave during silence.

## Open questions

- First view: use Sound once a sample is ready. Before that, retain the startup strip and its recording hint.
- Focus hold: use 150 ms for an automatic change. A pin applies at once. An explicit unpin returns to automatic focus.
- Rows and density: use three brightness levels and all available playback lines at the current 30 Hz UI rate. Allocate only the rows that have content.
- Playback position: use the last completed callback position. The device delay can differ across audio backends. A device delay readout and new refresh or density settings stay separate.
- Input: use Alt+O to change the view and Alt+P to pin. The first free panel cell is P16 on Pads. Use P16 in Pad Edit for the pin. From the visible panels each action takes one press. The pin route from the Pads panel also uses Edit and Back.
- Playback types: show the forward playback and fade paths the pad engine supplies. A pad which restarts starts a new line. Pad reverse and pad looping need their own sound changes. A loop preview on Chop stays in Chop.

## Checks

| Case | Expected result | Status |
| --- | --- | --- |
| Two slices share one sample | One wave shows both regions | Planned |
| Samples exceed available rows | The hidden count and pad labels stay visible | Planned |
| A voice restarts or ends between draws | The new voice uses its own position. The last wave stays in silence | Planned |
| The owner changes the project or replaces a sample | Old voice data does not select a sample in the new state | Planned |
| Edit or recording takes the strip | The original strip comes back on exit | Planned |
| 32 voices play with fades | The callback meets the 50 percent budget | Planned |
