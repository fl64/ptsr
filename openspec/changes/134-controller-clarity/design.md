# Design

## Owner decisions, 2026-11-19

The owner accepted the recommended options of the audit:

- Pad numbers: the number printed on the device everywhere, the keyboard key in the pad corner.
- Yes and No: P1 No, P4 Yes on all screens and in all questions.
- Hints: they follow the device of the last input.
- Target choice: all 16 pads on the playing layer. Commands only during a PAD BANK hold.
- The goal comes first: each step shows what happens, the next step and how to go back.

Audit evidence: `target/ux/134-audit/notes.md` and its screen captures.
The audit used a virtual port with PAD BANK `Off`. It did not check sound,
recording or the physical hold.

## Terms

- Device number: the number printed on the SMC-PAD. Pad 1 is bottom left, pad 16 is top right.
  The ptsr pad index stays as it is inside the program and in saved files.
- Control row: the bottom row of the panel, device pads 1 to 4.
- Last input device: the source of the last accepted key, pad hit, button or knob turn.
  A knob turn counts. A release does not count.
- Mode line: one line above the hints. It names the mode, the item or operation,
  the next step and the cancel control.

## Decisions

### D1. One pad number

While an SMC-PAD Master port has a connection, pad labels use device numbers.
Each bank shows pads 1–16. The side panel adds the bank number.
The keyboard key shows in the corner of each pad frame. Without an SMC-PAD the grid
keeps the keyboard keys. The side panel, messages, questions and Help use the same
number as the grid. Help draws the device with pad 1 bottom left.

Default for an open question: the bank prefix stays as now (`bank 2, pad 5`).

### D2. Control row

| Device pad | Normal view | Question | Target mode |
| --- | --- | --- | --- |
| 1 | back | no | cancel |
| 2 | undo | dim | undo |
| 3 | redo (dim until 93a) | dim | menu |
| 4 | dim | yes | apply (Assign) or dim |

Panic leaves the panel. Change 93a decides its new route. The plan proposes a
double STOP press. A dim cell runs nothing and says nothing.

### D3. Questions

A question answers with device pad 1 (No) and pad 4 (Yes) from either note bank,
without a hold. Other pads stay silent. PLAY and STOP keep their transport roles
and never answer. The question text names `SMC 1: <no word> · 4: <yes word>` only
when the last input device is the SMC-PAD. Otherwise it names keys. Back is the
same pad as No, so every question also closes with Back.

Risk: device pad 4 is a playing pad. A stray hit on pad 4 during a question
answers Yes. The current pad 12 has the same risk. Keyboard answers stay.

### D4. Target mode on the playing layer

When an item is in hand (file, recording, copy, swap) or an Assign plan is open:

- Without a hold, each of the 16 pads places the item, as a first-bank hit does now.
  The panel is hidden. The grid marks the source, the occupied pads and the planned slice numbers.
- During a hold, the panel shows the commands of the mode: control row as in D2,
  `order` for Assign in the top row, other cells dim.
- With PAD BANK `Off`, the panel shows the same commands. A first-bank hit hides the
  panel and places the item.

This change removes the current panel grid of 12 targets.

### D5. Hints by the last input device

The hint lines, the mode line, question answer rows and mode texts use the words of the last input
device. SMC-PAD words name device pads (`1`, `4`), buttons (`PAD BANK`, `LEFT`)
and knobs (`knob 7`). Keyboard words stay as now. A change of device redraws the
hints at once. F1 keeps both, as now.

### D6. Mode line

The mode line shows, for each current mode: the mode name, the item or target,
the next step and the cancel control. Examples after an SMC-PAD input:

- `IN HAND break-120 · hit a pad · PAD BANK + 1 cancel`
- `ASSIGN slice 4 of 9 next · hit a pad · PAD BANK + 4 apply · PAD BANK + 1 cancel`
- `● REC OUTPUT 0:04 · stop: PAD BANK + 6 output`
- `EDIT pad 5 · TONE · LEFT RIGHT page · PAD BANK + 1 close`

The line stays after the owner releases PAD BANK. It does not replace notices.

### D7. Honest panel and title

- The title keeps the screen name. The header above the cells names the panel view.
- Badges and the BPM keep their columns. Text that does not fit is cut at the end.
- During a question, the panel shows only `no` on pad 1 and `yes` on pad 4.
- While the panel shows, the hint for LEFT and RIGHT names the panel views.

### D8. Knob numbers on rows

A row that a knob turns shows that knob number. In pad edit TONE the rows read
`7 volume`, `5 pitch`, `3 mute`. The knob strip and the row use the same word.
The FX screen and the project list follow the same rule (`knob 1`, not `E1`).

### D9. Refusals and results

A panel action that cannot act gives a notice with the reason. Example: Learn on
Settings. A placement result names the pads, for example `slices 1–3 → pads 5, 6, 9`.

### D10. Labels with one meaning, 2026-11-19

The owner renamed only the labels that are unclear or have two meanings. A label can
have seven cells, so that `rec out` fits. The Chop region cells left the page: knobs 5
and 6 move the region. `whole` takes their place and restores the whole sound. The owner also removed the
other duplicates: Files `parent` and `enter` (LEFT and RIGHT do this) and Settings
`tab`, `row -` and `row +` (LEFT, RIGHT and knob 1 do this).

| Screen | Old | New |
| --- | --- | --- |
| Pads | take, clear, marks, gate, output | swap, remove, all, p.gate, rec out |
| Pad edit | clear, gate | reset, p.gate |
| Files | take, auto | to pad, listen |
| Chop | start, end, commit | removed, removed, fill |
| Chop edit | level | normal |
| A1 | knobs, undo | params, undoA1 |
| Settings | clear | unbind |

## Route table

Counts are presses with `Hold panel`. A hold counts as one press. Pad hits on targets
count. Before: the current build. After: this change.

| Scenario | Before | After | What changes for the owner |
| --- | --- | --- | --- |
| Answer a replace question | 1 (P11 or P12) | 1 (P1 or P4) | Same pads as Back and Apply |
| Leave Exit, Learn, list | 1 (P11) | 1 (P1) | Same pad everywhere |
| Load a file on pad 1–12 | 3 | 3 | Mode line and pad names show |
| Load a file on pad 13–16 | 4 | 3 | No layer switch |
| Assign 3 slices and apply | 6 | 6 | All 16 pads, slices shown on pads |
| Copy a pad | 3 | 3 | Source marked, result names the pad |
| Change Help tab | 1 (P11/P12) | 1 (LEFT/RIGHT) | P11/P12 no longer mean tabs |
| Find which knob turns volume | read strip | read row | Row and knob show the same number |

The full table of the 40 stories goes into `docs/CONTROLLER-PATHS.md`.

## Corner cases

| # | Case | Expected result | Status |
| --- | --- | --- | --- |
| E1 | A question opens while the owner holds a pad 4 Gate | The release ends that Gate and does not answer | test planned |
| E2 | Pad 1 and pad 4 hit in the same redraw during a question | Only the first answers | test planned |
| E3 | X-Session pad hit during an SMC-PAD question | Keeps its own route and does not answer | test planned |
| E4 | Last input changes from keyboard to SMC-PAD while a question shows | The answer row changes words. The question stays | test planned |
| E5 | Item in hand and a hold, then release without a hit | Item stays in hand. The mode line stays | test planned |
| E6 | Assign target on bank 2 pad | Plan keeps bank and pad. The grid shows the slice | test planned |
| E7 | SMC-PAD disconnects with an item in hand | Grid returns to keyboard keys. The item stays | test planned |
| E8 | Owner binding replaces a cell of the control row | Questions still take pads 1 and 4 | test planned |

## Open questions

1. Recording output stays on the Pads cell. The owner renamed it to `rec out` (D10).
2. A1 has `undo` in the action page and the control row. The owner renamed the A1 cell
   to `undoA1` (D10).
3. Labels with two meanings. The owner chose the renames in D10.
4. Pad numbers without an SMC-PAD. Default: keyboard keys, as now.
