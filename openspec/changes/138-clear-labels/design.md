# Design

## Owner decisions, 2026-11-20

- Every panel label says plainly what the press does. The owner accepted the label table below.
- The Chop knobs get clearer names too.

## Decisions

### D1. The label table

Each label has at most two words. The first space starts the second line (change 135), and
each line keeps at most seven cells. The table in the command panel delta is the full list.
Two labels changed from the first draft to fit seven cells: `A1 setup` and `star file`.

### D2. One label with two states

The pads toggle shows `hide pads` while the pads show, and `show pads` while they are hidden.
F1 Help names it «hide the pad grid, the wave takes its place». The owner did not understand
`grid` on 2026-11-20 until the agent named the hidden part.

### D3. The Chop knob labels

The knob strip keeps short names of at most six cells: `r.from`, `r.to` and `hits`. The line of
the last turned knob names them in full: «region start», «region end», «hit level».

### D4. Whole sample

On 2026-11-20 the owner pressed `whole` and saw nothing: the region already covered the whole
sample. The owner did not know what «whole file» meant, so the label is `whole sample`. The cell is
dim while nothing can change, and each press shows a notice, as `join left` does (change 135).

### D5. Dim cells

On 2026-11-20 the owner pressed cells that did nothing and asked to show only what works now.
A code check of every cell gave these conditions. A dim cell keeps its place and its label.

| Cell | Dim while |
| --- | --- |
| copy pad, chop pad, clear pad | the selected pad is empty or loading |
| reset pad | no pad in the scope holds a sound |
| steps wider / steps finer | the view is at the widest / finest step |
| edit steps, erase hits, clear A1 | A1 has no hits |
| undo A1 pass | the newest undo step is not an A1 step |
| take file, load to pad, chop file | the list has no file under the cursor |
| load to pad | the selected pad already plays this whole file |
| cut here | the cursor is outside the region or 10 ms from a cut |
| whole sample | the region is the whole sample |
| fewer hits | the threshold is −6 dB, or the chop has no cut by hits |
| join left / join right | the selected slice has no neighbour (change 135) |
| level to peak | Chop opened from a file, or the pad has no sound |
| forget control | the selected MIDI row has no learned control |

### D6. Destructive cells and questions

These cells use the warning colour: `clear pad`, `reset pad`, `clear A1`, `load to pad`,
`equal parts`, `cut by hits` and `fewer hits`. `forget control` and the threshold knob use it too.

- The cut actions ask only when hand cuts would go. The chop remembers each cut, move or join
  by hand. The next cut by parts or hits clears that memory. So repeated `cut by hits` presses do
  not ask again.
- `clear pad` asks for a pad with a sound. `reset pad`, `clear A1` and `load to pad` keep their
  existing questions.
- `forget control` asks and says that no undo exists. It then names the forgotten control.
- `level to peak` gets a pads undo step, so it needs no question.
- Each destructive action ends with a notice that names the change and the undo control.

## Open questions

- A1 recording replaces hits on the same tick while the owner plays. Default: no question,
  because it happens live. The pass notice already counts replaced hits.

- `rows` and `tap` in A1: the owner first asked to keep them, then asked to rename all labels. Default: rename to `hide empty` and `tap tempo`.
