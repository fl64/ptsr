# Design

## Sources

- The owner interview on 2026-11-20, questions Q1–Q28. Each decision below names its question.
- The external walk report of 2026-11-20: 51 findings (F01–F51), base `6be54ba`. Each decision
  names the findings that it closes.
- The owner approved one change for all items (Q18).

## Decisions

### D1. Four cell looks (Q4, F10, F16, F26, F28, F47, F48)

| Look | Meaning | A press |
| --- | --- | --- |
| Bright | The action works now. | It acts. |
| Orange | The action replaces or removes work. | It acts, or asks first (D6). |
| Grey | The action exists here, but cannot change anything now. | A notice says why and what to do first. |
| Empty | The cell has no action. | Nothing. |

The screens view and the colours of the bottom row stay as they are. A grey cell follows the
state at once. The conditions are in the D5 table.

### D2. Labels (Q5)

Labels are English and use up to two plain words. A label of two words always shows each word
on its own line. On 2026-11-20 the owner saw `copy pad` on one line and `pitch gate` on two
lines and asked if it was a bug (Q31). Each line uses at most eight cells, because `all cuts`
needs eight. One label has one meaning on all screens. The command panel delta has the full table.

The slice edit cell for normalize is `norm`: `normalize` has nine letters and no space (Q29).

### D2a. State cells (Q30, Q31)

On 2026-11-20 the owner could not see which play range was on. A cell that changes a state shows
the current state under its label, in another colour. The owner asked to keep the agreed
two-word labels. At 120×40 a cell has three lines: the two words and the state. At 80×26 a cell
has two lines of 11 cells. All state labels fit 11 cells, so the label takes one line and the
state takes the second. The command panel delta lists the cells and their states.

`hide pads` shows `show pads` while the pads are hidden. F1 Help names it «hide the pad grid,
the wave takes its place».

### D3. Layouts by frequency (Q6, Q7, Q8, Q9, Q10, Q11, Q13, Q15)

P1–P4 stay back, undo, redo and done on all screens. The other cells use printed pad numbers.

| View | P5–P8 | P9–P12 | P13–P16 |
| --- | --- | --- | --- |
| Chop | cut here, cut by hits, fewer hits, equal parts | put on pads, fill pads, edit slice, trim silence | remove all cuts, loop play, play range, cursor step |
| Slice edit | norm, join left, join right, hear as pad | — | — |
| Pads | edit pad, copy pad, chop pad, clear pad | swap pads, mark pad, mark bank, play mode | pitch gate, record output, hide pads, — |
| Pad edit | play mode, copy pad, pitch gate, reset pad | tone page, env page, filter page, source page | fx page, —, —, — |
| Files | load to pad, take file, listen file, chop file | star file, —, —, — | — |
| A1 | record A1, edit steps, erase hits, tap tempo | steps wider, steps finer, A1 setup, clear A1 | hide empty, —, —, — |
| Setup | choose, —, —, — | — | — |

- `fewer hits` acts on the whole sample, so it moves to Chop. `hear as pad` acts on one slice,
  so it moves into the slice edit.
- `undo A1 pass` leaves A1. P2 undo on Pads and A1 reaches the A1 steps (Q11).
- The panel file counts cells from the top left: cells 1–4 are P13–P16, cells 5–8 are P9–P12,
  cells 9–12 are P5–P8 (`tui::pads::device_number`).
- P16 on Pads stays free for a later microphone recording cell.

### D4. Trim instead of whole (Q2, Q14)

Samplers call it a trim: the sound loses its start or its end before the cuts. The screen uses
only the word trim. Knob 5 is `trim<` («trim start»), knob 6 is `trim>` («trim end»), knob 7 is
`hits` («hit level»). `whole` leaves the panel. To remove a trim, the owner turns knobs 5 and 6
to the ends. The words region and whole leave the panel, the knob strip and the notices.

`trim silence` moves the trim start to the first sound and the trim end to the last sound.
A sound is a sample louder than −50 dB below the peak of the file. This value is a default that
the owner checks by ear. Without cuts, it acts at once. With cuts, it asks first, because cuts in
the trimmed parts go and the slices of linked pads change (Q2.4). Undo brings back the trim and
the cuts.

### D5. Grey cells

A code check of every cell gave these conditions. A grey cell keeps its place and its label.

| Cell | Grey while | A press says |
| --- | --- | --- |
| copy pad, chop pad, clear pad | the selected pad is empty or loading | what the pad needs first |
| reset pad | no pad in the scope holds a sound | «nothing to reset» |
| steps wider / steps finer | the view is at the widest / finest step | the limit |
| edit steps, erase hits, clear A1 | A1 has no hits | «A1 has no hits yet» |
| erase hits, edit steps | the A1 count-in runs | «wait for the count-in» |
| take file, load to pad, chop file | the cursor is not on a file | «select a file» or «RIGHT opens the folder» |
| load to pad | the selected pad already plays this whole file | that it plays it |
| cut here | the cursor is in a trimmed part or 10 ms from a cut | why |
| fewer hits | the chop is cut by hits at −6 dB | «by hits stays at -6 dB» |
| remove all cuts | the chop has no cut | «no cuts to remove» |
| trim silence | the trim edges already sit on the first and last sound | «no silence to trim» |
| join left / join right | the slice has no neighbour (change 135) | the side |
| norm | no pad plays the selected slice | «put the slice on a pad first» |
| apply (Assign) | in order, and the start pad is not hit | «hit the start pad first» |
| undo | nothing to undo | «nothing to undo» |

### D6. Destructive actions (Q3, Q4, F05–F09, F41, F43)

Orange cells: `clear pad`, `reset pad`, `clear A1`, `load to pad`, `equal parts`, `cut by hits`,
`fewer hits`, `remove all cuts`, `trim silence` and the Assign method cell. The threshold knob
uses the same rule.

- The cut actions ask only when cuts made by hand would go. The chop remembers each cut, move
  or join by hand. The next cut by parts or hits clears that memory. So repeated `cut by hits`
  presses do not ask again.
- `remove all cuts` follows the same rule. It asks only when hand cuts would go.
- `clear pad` asks for a pad with a sound. `reset pad`, `clear A1` and `load to pad` keep their
  questions.
- A question names the item and the undo: «Replace 4 hand cuts with 2 equal parts? P2 undo
  brings them back.» A question says «This cannot be undone.» only when no undo exists.
- After the action, a notice names the result and the undo: «4 cuts replaced · P2 undo brings
  them back». After keyboard input, the undo is named by its key (D10).

### D7. Undo notices (Q22, F09, F11, F51)

Undo replaces the current notice with what came back. It uses the words of the action, for
example «undo: 5 slices back (were 9)» or «undo: slices 1 and 2 split again».
The rule applies on Chop, Pads and A1. With nothing to undo, the cell is grey (D5).
Redo stays inactive until change 93a, as the command panel spec says. The interview assumed a
working redo. The agent found this after the summary and told the owner.

### D8. Normalize (Q8.2)

On 2026-11-20 the owner asked for a level button that does not use the pad volume. The pad
volume (−40…+6 dB, `crates/app/src/pads.rs`) stays a mixing control.

- `norm` (normalize) sets the sound gain of each pad that plays the selected slice. The peak of what
  the pad plays goes to −1 dB. The gain can be negative.
- The gain is saved per pad in the project as an optional field. A project without the field
  loads with 0 dB. The format needs no migration.
- The engine applies the gain before the pad volume, with the usual smoothing. The audio thread
  gets the gain value through the existing pad parameter path. No allocation, no lock.
- A second press measures again and keeps the level. Undo on Pads brings back the old gain.
- The gain stays at +30 dB or less, so that a silent slice does not make noise loud. A notice
  says when the limit stopped it.
- Ctrl+G stays the key. The old pad volume behaviour leaves.

### D9. Binding changes from the keyboard only (Q20, Q21, F01, F02, F04, F30, F31, F47)

On 2026-11-20 the owner said that bindings with pads only are painful. All changes of control
bindings use the keyboard: F10 Learn, learn and forget in F12 MIDI, reset to profile and the
SMC-PAD layout update. The controller only gives the control to bind. This is the only
exception to the controller goal. It goes into `docs/plans/` and `docs/VISION.md`.

- The `learn` cell leaves the screens view. `learn control` and `forget control` leave Setup.
- Setup keeps one cell, `choose` on P5. It acts as Enter. It opens a group, steps the knob
  response or the hold time, changes the PAD BANK mode or opens the folder picker on PATHS. On a binding row
  (an action, a set, reset or the layout update) a press from the SMC-PAD says «Bindings: keyboard
  only (F10, Enter)».
- F12 MIDI says «Bindings: keyboard only (F10, Enter)».
- Other settings stay on the controller.
- Forget, reset and the layout update ask first and say «This cannot be undone.». They have no undo.

### D10. The mode line (Q23, F12, F14, F15, F19, F24, F25, F27, F32, F33, F35, F36, F44, F45, F46, F49)

- The line has four parts: «MODE · in hand · next step · exit». The exit part is always the last.
- At 80×26 the middle parts give way first. The exit part is never cut. Change 135 kept only
  two `PAD BANK +` parts. The rule now keeps the last part in all lines.
- The line uses the words of the last input device: «PAD BANK + 1 close» or «Esc close».
- New lines:
  - the slice edit: `EDIT slice 3 · BOUNDS · PAD BANK + 1 close`
  - A1: `COUNT 4` and `REC A1 · hit pads · RECORD stop`
  - the step edit: `STEP 5 · hit a pad to add or remove · PAD BANK + 1 close`
  - A1 setup: `A1 SETUP · knob 3 quantize · knob 4 strength · knob 5 bars · PAD BANK + 1 close`.
    The knobs come from `config/devices/smc-pad.toml`, `[screens.pattern]`.
  - tap tempo: `TAP · 1 hit · tap again`
  - erase: `ERASE A1 · … · PAD BANK + 7 off`
- Hold PLAY shows «metronome on» or «metronome off». The status shows `click`, not `M`.
- With PAD BANK set to Off, the idle line is «PAD BANK: hit to show commands».
- The copy line keeps the source: «COPY kick B1/P1 · hit a pad · PAD BANK + 1 cancel».
- Pad edit with the panel shown says «◀ ▶ view», without the panel «◀ ▶ page».
- Chop with slices on the grid says «◀ ▶ slice page», after an assign «◀ ▶ bank».
- The save failure question uses «Back» and «Retry save» in the question, the line and the cells.
- Help Controller names «PAD BANK + 5/6 select».
- Only the notices that the walk found change device words in this change: the output recording
  stop, the A1 erase result and the undo texts. About 270 other texts that name keys go to
  `docs/BACKLOG.md`.

### D11. P1 leaves the mode first (Q25, F50)

PAD BANK + P1 first leaves the active mode: erase, copy, swap, a file in hand or marking. The
screen stays. The next press is the usual back. A second press of `erase hits` also turns erase off.

### D12. STOP ends the output recording (Q24, F34, F35)

STOP stops the output recording on all screens. If A1 plays, the first STOP stops A1 and the
recording goes on. The line says «● REC OUTPUT 0:12 · STOP end rec». The second STOP saves.
If A1 does not play, one STOP saves. `record output` on Pads also stops a recording.

### D13. Pad edit pages (Q28, F13, F37)

The pad edit panel gets five page cells, P9–P13: `tone page`, `env page`, `filter page`,
`source page`, `fx page`. The current page is brighter. LEFT and RIGHT without PAD BANK go
around: LEFT on TONE opens FX.

### D14. Assign (Q26, Q27, F21, F22)

The method cell shows the current method: `by hand` or `in order`. With no slice placed, a
press changes the method at once. With slices placed, it asks «Switch to in order? 2 placed
slices go.». After yes: «in order · P2 undo brings 2 back». `apply` is grey in order until the
start pad is hit.

### D15. A new project with no projects (Q19, F03)

With no projects, PAD BANK + P4 `new project` in the project picker makes and opens a project
named by the date and a number: `2026-11-20-1`. The owner can rename it from the keyboard.

## Open questions

- Q24 option: the owner chose STOP but did not choose between one press (stop all) and two
  presses (A1 first). Default: two presses (D12).
- Normalize gain limit: default +30 dB (D8).
- The silence level of `trim silence`: default −50 dB below the peak (D4).
- A1 recording replaces hits on the same tick while the owner plays. Default: no question,
  because it happens live. The pass notice already counts replaced hits.
- The keyboard trim keys `,` `.` and `Ctrl+,` stay. The decision to remove `whole` covers the
  panel only. The controller route to clear a trim is knobs 5 and 6 to the ends.
- `pattern.undo` keeps its key. Its controller route is P2 undo, which reaches the A1 step.
- STOP during the output recording first stops what plays: A1 or a preview. The next STOP saves
  the recording. That STOP does not reset the transport position.
- `reset pad` sets the sound gain to 0 dB, as it resets the other pad parameters.
- While learning, the SMC-PAD pad 4 no longer saves a choice, because a touched control can be
  pad 4. Enter saves and Esc keeps, from the keyboard.
- `fewer hits` follows Shift+H. The main spec («Reverse hit threshold») says that the first
  Shift+H without a cut by hits cuts by hits at −6 dB. So the cell acts there too, and it is
  grey only at −6 dB. One action has one result on both inputs.
