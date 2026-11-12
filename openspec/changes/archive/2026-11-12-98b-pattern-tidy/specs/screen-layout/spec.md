# screen-layout Delta

## ADDED Requirements

### Requirement: Strip header explains multiple hits

When a visible cell of the selected pad holds two or more hits, the strip header SHALL end with `2-9 = hits in a cell, one plays`. Without such a cell, the header SHALL NOT show it. When the width is too small, the legend SHALL be cut before the bank.

#### Scenario: Strip header explains multiple hits
- **WHEN** the visible page of the selected pad has a cell with two hits, and then has none
- **THEN** the header ends with the legend only while such a cell is visible
- **AND** the bank stays visible at 80 columns

### Requirement: Whole-pad erasure shows hit counts

In whole-pad erasure, the Pads title SHALL show `ERASE`. Each pad of the open bank with A1 rows SHALL show `N hits` on its mode line. N SHALL count every raw row that a press erases, hidden rows included. A pad without rows SHALL show an empty mode line. After an erasure, the count SHALL go away at the next redraw. Leaving erasure SHALL restore the usual mode lines. The counts SHALL fit 80x26 and 120x40.

#### Scenario: Erasure shows what a press erases
- **WHEN** whole-pad erasure is on with 4 hits on pad Q, 2 hits on pad W and none on pad E, at 80x26 and 120x40
- **THEN** the title shows `ERASE`, pad Q shows `4 hits`, pad W shows `2 hits`, and pad E shows no count
- **AND** after a press on pad Q its count goes away, and leaving erasure restores the mode lines

### Requirement: Pattern controls take the side panel

While pattern controls are open, the right column SHALL show only the A1 controls in the style of the pad panel. Block lines SHALL name `A1 PATTERN` and `CLEAR`. Labels SHALL be lowercase, and the selected row SHALL be bright. The keys SHALL show dim at the bottom of the column. The pad panel SHALL NOT show under the controls. Closing SHALL restore the screen's own column. Title, bank, pad grid, strip and help SHALL keep their positions.

#### Scenario: Pattern controls replace the pad panel
- **WHEN** the owner opens pattern controls on Pads, Chop, Files and FX at 80x26 and 120x40
- **THEN** the right column shows the A1 PATTERN and CLEAR blocks and the keys, and no pad parameter
- **AND** closing restores the pad panel and every other position stays

### Requirement: Pattern controls use the pad edit frame

Pattern controls SHALL use the frame of the pad edit: the edit color and the title `EDIT ·` with the block names `PATTERN` and `CLEAR`. The block of the selected row SHALL be lit, as the open page of the pad edit is. A narrow column SHALL drop `EDIT ·` before the block names, as the pad edit does.

#### Scenario: Pattern controls use the pad edit frame
- **WHEN** the owner opens pattern controls and selects Bars, then Clear A1, at 80x26 and 120x40
- **THEN** the column has the edit frame with `PATTERN` lit, then with `CLEAR` lit
- **AND** the frame has the color of the pad edit frame

### Requirement: Pattern controls show restrictions in place

The edit restriction of the selected row SHALL show dim next to its value. A refused edit SHALL show the restriction in the warning color.

#### Scenario: Pattern controls show restrictions in place
- **WHEN** the owner selects Strength, then tries to change Bars during Recording
- **THEN** `step 10` shows dim next to Strength, and the Bars refusal shows in the warning color

## MODIFIED Requirements

### Requirement: Pattern strip identifies time and event density

A word header SHALL identify A1, the playback bar and beat, the loop length and the open bank. It SHALL name the view division as `view`, and the quantize and strength of the next hits as `quantize`. The selected pad's row SHALL have the label `pad` with the pad key, for example `pad Q`. The strip SHALL NOT show the pad file name. The all-pads row SHALL mark cells where any pad of the current bank has an onset. Empty, aligned, off-grid and multiple-event cells SHALL have distinguishable forms, with dim empty cells. Same-tick rows SHALL remain visibly multiple. Drawing SHALL NOT change raw rows or settings.

#### Scenario: Pattern strip distinguishes subdivisions and event density
- **WHEN** straight and triplet pages contain aligned hits, between-line hits, collisions, empty cells and hits of other pads in the bank
- **THEN** their forms, beat backgrounds, bar lines, header words and the all-pads row identify the corresponding data
- **AND** drawing changes no raw row, velocity, duration or setting

#### Scenario: Strip header names view and quantize apart
- **WHEN** the view is 1/16, the quantize is 1/4 at 100 %, and the owner selects pad Q of bank 4 with a long file name
- **THEN** the header shows `view 1/16 · quantize 1/4 100 % · bank 4`, and the pad row label is `pad Q`
- **AND** the file name does not show in the strip

### Requirement: Pattern strip yields to overlays and keeps safety visible

Pad-parameter curves and audio recording SHALL keep their use of the strip, and the event rows SHALL NOT draw then. The splash SHALL use only the wave rows above the event rows. A strip without wave rows SHALL show the event rows and no splash. Save, audio and xrun warnings SHALL stay visible with the event rows at both supported sizes. F2 SHALL keep terminal-cell drawing. Only Chop collects kitty images.

#### Scenario: Pattern strip yields to overlays and keeps safety visible
- **WHEN** F2 shows A1 with a pad-parameter curve, audio recording, or save and audio warnings
- **THEN** curves and the recording wave take the strip without event rows
- **AND** warnings stay visible next to the event rows at 80x26 and 120x40

#### Scenario: The splash leaves the event rows visible
- **WHEN** the program starts with and without A1 at 80x26 and 120x40
- **THEN** the event rows or the recording instruction show at both sizes
- **AND** the splash shows in the wave rows at 120x40, and a three-row strip at 80x26 shows no splash

