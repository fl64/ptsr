# Plan: visual control — badges, the status line, messages (change 87 and rules for later changes)

Read `README.md` first. The goal: at one look, without reading, the owner knows **what the pads
do now, what plays, what records, what waits, and what the last press did**, on every screen and
from any input. This is most important with the controller: the hands are on the SMC-PAD and the
eyes only glance at the screen.

## 1. What the screen has now

| Element | Where | What it shows | Problem for the goal |
|---|---|---|---|
| Screen badge | title line, left, inverse (`F3 CHOP`) | the screen | good, keep |
| Mark plate | title line, yellow on dark (`MARK · hit pads · 3 marked`) | marking on / pads marked (76) | the only plate, a one-off function `mark_plate`; other modes have no plate |
| Title text | title line | per screen: file in hand, chop mode, slices | free text per screen; a mode can hide in a sentence |
| Status line | `tui::Status`, `status_line` | `● rec`, backend, buffer, CPU, master, xruns, no-releases warning, project | no transport, no input devices, no layer, no pattern |
| Hint line | `hint_for` in `main.rs` | the next step, by screen | names keyboard keys only ("Enter takes a file", "J L") |
| Flash | `App.flash: Option<(String, Instant)>`, set in about 68 places | the result of an action, errors | one slot, no level (an error looks like "saved"), a new message hides the old one, no history |
| Startup notices | `App.startup` | config, session, terminal problems | shown once, gone at the first key |
| Lit value | `App.lit` | what a knob or a value key changed last | only where the pad panel or the slice line is visible |
| Help line | bottom | keys by groups, modifier-aware | keyboard only |

## 2. Target

Four zones with fixed meanings. Nothing else shows state.

```
 F2 PADS  [STEP]  [● REC]  [▶ A1 2.3]                    ← 1 title line: screen badge + up to 3 badges
 ┌─ strip ───────────────────────────────────────────┐
 │                                     decay 240 ms   │   ← 2 value pop-up (top-right of the strip)
 └────────────────────────────────────────────────────┘
 bank row / pad grid (pad glyph badges) / right column
 ✓ slices on pads 1–8                                     ← 3 message line
 ▶ 120.0 · A1 2.3 │ ● REC │ KB · SMC L2 │ jack 128 · CPU 12 % · xruns 0 │ my-track   ← 4 status line
 [help line by device]
```

### 2.1 Badges in the title line

One model, one function:

```rust
pub struct Badge { pub text: String, pub kind: BadgeKind, pub priority: u8 }
pub enum BadgeKind {
    /// The pads do something other than sound: yellow on dark (the mark plate style).
    PadMode,
    /// Something is on / plays: cyan.
    On,
    /// Waits, records, asks, warns: magenta.
    Attention,
}
fn badges(app: &App) -> Vec<Badge>   // in crates/app; the tui draws them
```

Rules:

- `badges` reads `App::focus()` (change 85) and the transport state. It is the only place that
  decides badges. A screen title keeps its own text (file, slices) but never a mode.
- Not more than 3 badges; sorted by priority; the pad-mode badge is always first when it exists.
  When the line is short, badges with lower priority drop first, then their text shortens to a
  short form (`MARK 3`, `STEP`, `REC`).
- Each badge has text. Color alone never carries a meaning (some terminals have poor colors).

Badges by state (later changes add theirs; the table in the spec is the source):

| State | Badge | Kind | From change |
|---|---|---|---|
| marking on / pads marked | `MARK · hit pads · 3` / `3 MARKED` | PadMode | 87 (moves the 76 plate) |
| file in hand | `IN HAND · hit a pad` | PadMode | 87 |
| pad in hand (swap) | `SWAP · hit a pad` | PadMode | 87 |
| chop assign | `ASSIGN · 5/8 placed` | PadMode | 87 |
| edit open | `EDIT · TONE` (page name) | On | 87 |
| learn on the screen | `LEARN` | PadMode | 87 |
| question open | `ASK` | Attention | 87 |
| recording mic / output | `● REC 4 s` | Attention | 87 |
| chop loop / as pad | `LOOP`, `AS PAD` | On | 87 |
| command panel | `PANEL` | PadMode | 92 |
| transport plays | `▶ A1 2.3` | On | 95, 96 |
| count-in | `COUNT 3` | Attention | 97 |
| pattern recording | `● REC A1` | Attention | 97 |
| step / erase | `STEP`, `ERASE · hold a pad` | PadMode | 98 |
| slot pick / queued | `SLOT`, `NEXT A2` | PadMode / Attention | 99 |
| note repeat | `REPEAT 1/16` | PadMode | 100 |

The yellow color now means "the pads do something other than sound", not only "marked". This
changes `docs/ui/STYLE.md` (one line in the table and one rule). Ask the owner (README).

### 2.2 Pad glyph badges and grid tint

A pad cell already shows fill, frame, marks and the hit flash. Add one corner glyph per pad, in
the color of the current pad mode:

| Mode | Glyph on a pad | Meaning |
|---|---|---|
| in hand / swap | `◎` on empty pads, `!` on occupied pads | a hit puts it here; `!` asks before replacing |
| assign | the slice number planned on the pad | as now on the chop strip, now also on the pad |
| step | `▮` when the selected row has a step here on this page | the 16 pads are the 16 steps |
| erase | `×` on pads with hits in the pattern | a held pad erases |
| slot pick | slot state: `·` empty, `■` has hits, `▶` plays, `»` queued | |
| panel | the cell label in place of the sound name | from change 92 |

While a pad mode is on, the frame of the whole grid takes the mode color. The owner sees the
change in the side vision, not only in the title line.

### 2.3 Message line

Replace `App.flash` with a small model:

```rust
pub enum Level { Done, Info, Warn, Error }
pub struct Notice { pub level: Level, pub text: String, pub at: Instant, pub from: Source }
pub enum Source { Keyboard, Device(String), System }
fn notify(&mut self, level: Level, text: impl Into<String>)
```

- Glyph and color by level: `✓` Done (cyan), `·` Info (muted), `!` Warn (magenta),
  `✗` Error (magenta, bold).
- Times: Done and Info 3 s; Warn 6 s; Error stays until the next action of the owner.
- A new message of a lower level does not hide a living message of a higher level; it waits.
- The last 50 notices are kept. F1 has a "Messages" page; the panel has no cell for it (F1 is
  enough, see change 93).
- Startup notices go into the same model as Warn / Error, so they are in the log too.
- All about 68 `flash` sites move to `notify` with a level. Read each text again: plain words,
  short, English (STYLE.md).

### 2.4 Action echo for the controller

When an action comes from a device, the message line shows for 1.5 s what was pressed and what
it did, as Info: `SMC  STOP+Assign → plan applied`, `SMC  RIGHT → bank 3`. Without it, a
wrong press on the controller is invisible. Keyboard actions do not echo (the owner sees his
fingers). A setting turns the echo off.

### 2.5 Value pop-up

Every value change by a knob or by J / L shows the object, the parameter and the value at the
top right of the strip for `LIT_FOR`, on every screen: `pad W · decay 240 ms`,
`FX 1 · size 60 %`, `tempo 121.0`. With a short bar for 0..max. This is the same `App.lit`,
drawn in one place for all screens. The current local highlights stay.

### 2.6 Status line segments

Fixed order, left to right: transport, record, input, audio, project. When the line is short,
segments drop in this order: project, the audio details (`xruns` stays when it is not 0),
input. Transport and record never drop.

| Segment | Content | From |
|---|---|---|
| transport | `▶ 120.0 · A1 2.3` or `■ 120.0` | 95 |
| record | `● REC` mic / output / pattern | 87 (moved from the front) |
| input | the connected devices: `KB · SMC · XS`; the device of the last input lit cyan for 1 s; `L2` / `K2` when the last SMC-PAD note / CC came from the second pad / knob layer | 87 for devices, 92 for layers |
| audio | backend, buffer, CPU, xruns (as now) | as now |
| project | the name | as now |

### 2.7 Hints and help follow the device

`hint_for` returns a hint for the device of the last input: with the keyboard, the keys; with a
controller, its controls (`STOP+Take, then hit a pad`). Hints get the control names from the
same tables as the bindings (keymap, profile, `panel.toml`), never typed by hand, so a rebind
changes the hint.

The help line shows the 5 SMC-PAD buttons as one group `SMC` only while that controller is
connected. With only the keyboard, nothing changes in the help.

### 2.8 Questions

A question shows the `ASK` badge, its lines as now, and the answer controls for every connected
input: `Enter / STOP+Apply: yes · Esc / STOP: no`.

## 3. The changes

### Change 87 — `87-status-badges` (after 86, before 88)

Goal: §2.1, §2.2 (for the existing modes), §2.3, §2.5, §2.6 (without transport and layers),
§2.8. No change to keys or to what actions do.

Spec deltas: `screen-layout` (four zones, the badge line, the value pop-up), `screen-help`
(message levels, times, the Messages page in F1, the question answer line), `pad-playback`
(the mark plate becomes a badge; the pad glyphs for in hand, swap, assign), `docs/ui/STYLE.md`
(yellow = pad mode).

Tasks:

1. `Badge`, `BadgeKind`, `badges(&App)`; move `mark_plate` into it. Tui test per badge (the
   tui crate has buffer tests: follow `the_mark_plate_is_yellow_on_the_title_line`).
2. Pad corner glyphs and the grid frame tint for in hand, swap, assign, marking, learn.
3. `Notice`, `Level`, `notify`, the queue rules, the log; move every `flash` site. A test per
   rule (a Done does not hide an Error; an Error stays until the next action).
4. F1 Messages page.
5. The value pop-up in one place for all screens.
6. Status line segments: record, input (devices list and the last-input light), audio,
   project. The `Source` of each input event goes with it from the MIDI thread.
7. Question answer line with the controls of the connected inputs.
8. The screen catalogue `task screens` (`CROSS-CUTTING.md` §1.2).
9. The details of `CROSS-CUTTING.md` §6: kitty images under overlays, short labels, redraw
   only on change. Move the view model into `app/src/view.rs` as a separate commit (§5).

How to verify (owner): mark pads, take a file, start assign on Chop, open the pad edit: each
time the badge and the grid frame change at once; trigger a save and a load error: the error
stays and is in F1 Messages; turn a knob on any screen: the pop-up names the value.

### What later changes must add (rule for README)

Every change that adds a mode or a state adds, in the same change: its badge (table in §2.1),
its pad glyph if the pads change meaning (§2.2), its notices with levels, its hint for the
keyboard and for the controller (§2.7), and tui tests for them. Specifically:

- 92 command panel: `PANEL` badge, the panel grid, the `L2` / `K2` layer marks, action echo
  (§2.4), device-aware hints and the `SMC` help group (§2.7).
- 95 transport: the transport segment and `▶` badge.
- 97 pattern record: `COUNT n`, `● REC A1`, notices for the hit limit and dropped hits.
- 98 pattern screen: `STEP`, `ERASE`, step and erase glyphs.
- 99 slots: `SLOT`, `NEXT A2`, slot glyphs, the blink of the queued slot.
- 100 note repeat: `REPEAT 1/16`.
