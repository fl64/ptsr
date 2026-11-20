# Proposal

## Why

On 2026-11-20 the owner did not understand which bound `cut -` removes. The owner asked that
every panel label say plainly what the press does. On the same day, an external walk of all
SMC-PAD actions found 51 problems and 4 dead ends (base `6be54ba`). Cells did nothing and
looked active. Destructive actions did not warn. Undo worked but the screen did not say so.
Mode lines lost their exit at 80×26, and hints named keys after SMC-PAD input.

The owner decided all items in one interview (Q1–Q28) and approved one change for all of them,
as an explicit exception to the one-change-per-subject rule.

## What Changes

- Panel cells have four looks. Bright acts now. Orange changes or removes work. Grey cannot act now, and a press says why. Empty has no action.
- Every action label uses plain words, up to two words. A long label breaks at its first space. A label line may use eight cells.
- New layouts for Pads, Pad edit, Chop, slice edit, Files, A1 and Setup, ordered by how frequently each action is used. The bottom row P1–P4 stays back, undo, redo and done on all screens.
- Chop loses `whole`. Knobs 5 and 6 (`trim<`, `trim>`) trim the sound. The new cell `trim silence` moves the trim edges to the first and the last sound.
- Chop gets `remove all cuts` on P13 and on the Delete key.
- `fewer hits` moves from the slice edit to Chop. `hear as pad` moves into the slice edit.
- `normalize` in the slice edit sets a sound gain of the pad, so that the peak is at −1 dB. The gain is not limited by the pad volume range. It is saved in the project and undo removes it.
- Actions that replace cuts made by hand ask first. Cuts made by the program are replaced without a question. `trim silence` asks when the chop has cuts.
- After a destructive action, a notice names the result and the undo. Questions do not say «This cannot be undone.» when undo exists.
- After undo, the notice says what came back. Undo is grey when nothing can be undone. Redo stays inactive until change 93a.
- The mode line always shows the mode, the item in hand, the next step and the exit. The exit is never cut at 80×26. Lines are added for the slice edit, the A1 count-in and recording, the step edit, A1 setup, tap tempo and the metronome.
- The mode line and the notices found by the walk use the words of the last input device.
- PAD BANK + P1 first leaves the active mode and stays on the screen. A second press goes back.
- STOP stops the output recording on all screens. If A1 plays, the first STOP stops A1 and the second STOP saves the recording.
- Pad edit gets five page cells, and the arrows go around the pages.
- Assign shows its method `by hand` or `in order`. A method change asks when slices are placed. `apply` is grey until the start pad is hit.
- With no projects, `new project` in the project picker makes and opens a project with a date name.
- Changes to control bindings are done from the keyboard only. Learn and forget leave the panels. This is an explicit exception to the controller goal.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `command-panel`: cell looks, labels, layouts, questions and undo notices.
- `chopping`: trim silence, remove all cuts, normalize, hand cuts, fewer hits.
- `screen-layout`: Chop knob labels and the mode line.
- `controller-reach`: exit from modes, the binding exception.
- `midi-learn`: binding changes from the keyboard only.
- `recording`: STOP ends the output recording.
- `patterns`: A1 cells, erase, tap and count-in feedback.
- `pad-assignment`: the Assign method and apply.
- `pad-playback`: pad edit pages and the sound gain.
- `projects`: a new project from the controller.
- `transport`: metronome feedback.

## Impact

Sound changes only for `normalize`: the pad gets a gain stage before its volume. The project
file gets one new optional field per pad. Old projects load with a gain of 0 dB. No new
dependency. The change needs changes 135 and 136 first.

The change is large. The owner check is a full walk of about one hour.

Owner checks, controller:

1. `CHOP` → hold [PAD BANK] → bottom row back, undo, redo, done. P5–P8 «cut here», «cut by hits», «fewer hits», «equal parts».
2. `CHOP` → cut by hand → [PAD BANK]+[P8] «equal parts» → «Replace 4 hand cuts with 2 equal parts?» → [P4] → «… · P2 undo brings them back» → [PAD BANK]+[P2] → «undo: 4 cuts back».
3. `CHOP` → [PAD BANK]+[P12] «trim silence» → the grey parts cover the silence at the start and the end.
4. `CHOP` → hit a slice → [PAD BANK]+[P11] «edit slice» → [PAD BANK]+[P5] «normalize» → the pad plays louder → [PAD BANK]+[P2] → the level comes back.
5. `PADS` → empty pad → hold [PAD BANK] → «copy pad», «chop pad», «clear pad» are grey → [PAD BANK]+[P6] → the notice says why.
6. `PADS` → [PAD BANK]+[P14] «record output» → `A1` → [PLAY] → [STOP] → A1 stops, the recording goes on → [STOP] → «output saved».
7. `A1` → [PAD BANK]+[P7] «erase hits» → [PAD BANK]+[P1] → «erase off», the screen stays `A1`.
8. `PADS` → [PAD BANK]+[P5] «edit pad» → [PAD BANK]+[P13] «fx page» → the page is FX. At 80×26 the line ends with «PAD BANK + 1 close».

Owner checks, keyboard: F10 Learn and F12 MIDI learn, forget and reset ask as before.
Delete on Chop removes all cuts after a question.

Renders: `renders/138-normalize-before.wav` and `renders/138-normalize-after.wav`. The second
file is louder, with the peak at −1 dB and no clipping.
