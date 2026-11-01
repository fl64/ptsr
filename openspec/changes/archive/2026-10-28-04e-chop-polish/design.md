# Design 04e

## Help line

The help line takes one or two rows: groups are laid out in order and a group that does not fit on the first row starts the second. The screen body gets one row less only when the second row is needed. The chop screen lists its main step first: listen mode `[Enter] to pads`, assign mode `[Enter] apply`.

## Side panel

Lines of the side panel wrap at the panel width; a long summary takes two or three rows.

## Tab after a restart

A chop restored from the session does not know its last equal-parts count. If its number of slices is one of 2, 4, 8, 16, 32, Tab goes on from it; otherwise Tab starts at 2, as before.

## Small texts

- Listen mode: the pad of the selected slice is drawn as selected when that slice is on the current page.
- Overwrite question line: "bank B, pad K: what → slice N"; for a file: "bank B, pad K: what → file".
- Assign mode, empty pad: the second line is blank.
- One slice: "pad 1 plays the whole sample".
