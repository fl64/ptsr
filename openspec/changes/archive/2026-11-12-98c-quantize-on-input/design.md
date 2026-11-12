# Design

## Decisions

- **Quantize on capture.** The audio thread places the tick of an accepted onset with the grid and strength of the last published pattern. So a settings change between the hit and its collection by the App does not change the hit. The rounding rules of 96 stay in one function, `Grid::place`: the nearest grid midpoint chooses the later line, and a fractional tick rounds half later. The callback does one integer placement per captured onset, without allocation.
- **Playback plays saved ticks.** A saved pattern (`RawPattern::saved`) carries the grid and strength for the next hits but compiles its rows without a move. The App erasure check compiles saved ticks the same way, so a strength below 100 % does not move a row twice.
- **One hit per pad and cell.** The replacement rule of 97 uses the saved tick. Two hits of one pad on one saved tick keep the last captured hit. This also applies inside one pass.
- **Seam.** A hit that quantizes to the loop end saves tick 0. The fresh-hit rule of 97 still decides its first sequenced lap.
- **Gate length.** Quantize moves the onset only. The Gate duration stays as captured.
- **Old projects.** The owner rejected a conversion in the program. A saved tick plays as it is. A one-time script converts the owner's projects: each visible row gets the tick that 96 playback compiled for it. A hidden row moves the same way only while it stays hidden and in the tick range. Rows on one pad and tick all stay. The script keeps `project.toml.before-timing-<stamp>` next to each changed file.
- **Settings text.** The control rows keep their names. The selected row hint says `next hits`. F1 says that a change does not move saved hits.
- **Erasure and view.** Erasure and the pattern view use the saved ticks. They need no change.

## Open questions

- A command that quantizes all of A1 again. Default: not in 98c.
- The text of the `next hits` hint at 80 columns. Default: `next hits`.
