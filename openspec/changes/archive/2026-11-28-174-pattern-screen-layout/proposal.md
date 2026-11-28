## Why

The owner used the pattern work at `97cdda8` and found it unclear. F5 is a mode of Pads.
The overview has four dense header lines and a frame for each bank. The grid is small and
starts near column 46. Soft, medium and loud hits look almost the same. The `~` and `2`
marks have no explanation on the screen. The owner approved a mockup of a new screen.
This change gives F5 the approved look. It keeps every current key and SMC-PAD route,
so nothing that works today stops working. Later changes replace the setup panel (175),
add step input (176), finish the SMC-PAD model (177) and change Help (178).

## What Changes

- F5 opens the Pattern screen with the title `F5 PATTERN`, in place of the pad grid and the pad inspector. F2 and Esc go back to Pads. Owner keymaps under `[screens.pads]` keep working.
- One header line shows the transport state and position, the length, the tempo, the bar page and the mode badge.
- The rows are the pads with hits and the selected pad, in bank order, without bank frames.
  The pad label has its bank tint. A thin rule separates two banks. Alt+H adds all rows of the selected bank.
- The step cells use the full width. Each cell is at least three terminal cells wide. One page shows one bar at 1/16.
- Three velocity glyphs `▪` `▓` `█` show soft, medium and loud. A velocity lane shows the selected row.
- A detail line names the selected step, the pad and the cell content.
- A setup line shows bars, tempo, view grid, record quantize and strength, metronome and `count-in 1 bar`.
  It only shows values. `Alt+N` and the SMC-PAD `A1 setup` cell still change them.
- An absent or empty pattern shows `Ctrl+R records what you play` and the step edit or setup control.
- Step edit (F7), live record, count-in cancel, erase, tap tempo and the view keys work on the new screen.
- The Pads strip keeps its pattern lane. Without a pattern, it shows the hint `F5 pattern`.
- The Pattern screen, the Pads strip and the screens view say `pattern`, not `A1`.
- The old overview look and its catalogue entries go away.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- patterns: the Pattern screen rows, cells, pages, velocity, header, detail and setup lines.
- screen-layout: the Pattern screen layout, states and catalogue. The Pads lane hint.
- screen-help: Pattern screen help, the F5 title and the cues on the new lines.
- keyboard: F5 opens the Pattern screen. Current pattern keys keep their roles. Alt+H adds rows.
- midi-learn: learned `screen.pattern` and the all-rows button.
- command-panel: the screens view names `pattern`.

## Catalogue files

Review these files at 120x40 and 80x26 (`renders/screens/<name>-<size>.ansi`).
The Pattern screen is a Pads mode inside the program, so the names keep the pads prefix:
`pads-pattern-overview-absent`, `pads-pattern-overview-empty`, `pads-pattern-overview-straight`,
`pads-pattern-overview-triplet`, `pads-pattern-overview-off-grid`, `pads-pattern-overview-multiple`,
`pads-pattern-overview-velocity`, `pads-pattern-overview-hidden-hit`, `pads-pattern-overview-safety`,
`pads-pattern-step-straight`, `pads-pattern-step-cursor`, `pads-pattern-step-target` and `pads-pattern-step-safety`.
Also compare `pads-pattern-view-*` for the `F5 pattern` hint.

## How to check

Use a two-bar pattern with a kick on pad 1, a snare on pad 2 and a hat in bank 2.
Repeat at 80x26 and 120x40.

### Keyboard

1. `PADS` → F5 → `PATTERN` opens. The header is one line. Check «F5 PATTERN».
2. Check the rows: kick, snare and the hat, with a thin rule before bank 2. No frames.
3. Alt+H → all 16 rows of bank 1 show. Alt+H → the used rows return.
4. J / L → the detail line names the step, the pad and «loud (vel 127)» or «no hit».
5. Alt+N → the setup panel opens. Change bars. Esc → the setup line shows the new bars.
6. Ctrl+R → «COUNT» in the header → hit pads → Ctrl+R. The header shows the pass state.
7. F7 → «STEP EDIT» in the header → hit a pad → Ctrl+Z → Esc.
8. F2 → `PADS` with the same bank and pad. Clear the pattern → the lane shows «F5 pattern».

### SMC-PAD

1. [PAD BANK] + screens view → [pattern] → `PATTERN` opens.
2. Turn E1 → the selected column and the detail line move. [LEFT] / [RIGHT] → bar pages.
3. [PAD BANK] + [A1 setup] → turn the setup knobs → [P1] Back → the setup line shows the values.
4. [RECORD] → count-in → hit pads → [RECORD]. The header shows the pass state.
5. [PAD BANK] + [edit steps] → hit a pad → [P2] Undo → [P1] Back.

### Sound

The change needs no new WAV. It changes only the screen and the screen key. Playback, recording and saved hits do not change.

### Walk level

Run a targeted walk of the Pattern screen and the Pads lane (10–15 minutes).
