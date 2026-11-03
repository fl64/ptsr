# Plan: refactor of the input layer for controller-first work (changes 84–90)

Read `README.md` in this folder first. This document says what is wrong now, what the target
is, and what each change does. Line numbers are approximate; search for the names.
The review notes in this folder's README qualify the design sketches below. They are not
Rust interfaces to copy without checking the current implementation. Keymap cleanup 83a
(`KEYMAP-CLEANUP.md`) comes first; the unchanged-key reference for this track is its result.

## 1. What is wrong now

### 1.1 Two copies of "what does a pad hit mean"

- A keyboard pad goes `Keymap::resolve_in` → `App::run` / `App::apply` (UI thread). A chain of
  match guards there decides: sound, slice, plan, in hand, mark, question.
- A controller pad is decided in the **MIDI thread** (`crates/app/src/midi.rs`, the loop that
  calls `profile.resolve_in`). It reads a snapshot of five atomics in `midi::Shared`: `bank`,
  `screen`, `hand`, `edit`, `learn`. The UI thread writes them once per frame
  (`main.rs`, the block that does `shared.screen.store(...)`). The MIDI thread sends the sound
  straight to the engine (spec "Controller pads without delay") and then sends
  `Event::PadHit` to the UI. `App::midi_event` decides again, with **its own** match guards.
- The two chains already disagree. On the chop screen a keyboard pad goes to `App::chop_pad`:
  listen mode plays the real pad and selects its slice, assign mode puts the pad into the plan.
  A controller pad goes to `App::play_slice(abs % 16)`: it only plays a slice preview. So the
  assign plan cannot be filled from the controller, and listen mode is different. Change 84
  must add paired Chop scenarios through raw MIDI routing, not only direct App events.
- Every new mode (command panel, step mode, erase) would need edits in both chains.

### 1.2 Modes have no name

The state "what happens now" is spread over separate fields of `App`: `ask`, `plan`,
`pad_in_hand`, `browser.in_hand`, `pads.marking`, `edit`, `path_picker`, `project_picker`,
`pad_naming`, `browser.editing`, `learning`, `screen_learn`, `midi_run`, `help_open`, `splash`.
Their priority is the order of the guards in `run`, `apply` and `midi_event`. Nothing can say in
one value what the pads and the buttons do now. The screen cannot show it. A test cannot list it.

### 1.3 Knobs only know "go to a position"

`Resolved::Param` carries `value: 0..1`. Every `Param` treats it as an absolute position:
cursor, zoom, threshold, region edges, and even the edit page (`Param::Page`: the travel split
into pages). Endless encoders need "turned by N steps". Lists (files, projects, settings rows)
have no knob at all, only `BrowserUp` / `BrowserDown` and `SettingsRow`. The profile parser
reads `type = "relative"` and ignores it (`input/src/midi.rs`, comment "relative encoders
(SMC-PAD) are not supported yet").

### 1.4 Buttons only know "pressed"

A button acts on press (`value > 0`). Release only matters for pads and `RecordMic(false)`.
There is no tap / hold and no modifier button.

### 1.5 There is no "confirm" action

A question (`Ask`) is answered by the action of its own step: `ChopCommit | ChopEdit |
BrowserHold | PadEdit`, and `SettingsLearn` for the profile reset. On the keyboard this is
always Enter. On a controller each of these must be bound, and they mean other things on other
screens.

### 1.6 The screen list is in many places

`input::Screen`, the codes 0..4 in `main.rs` and again in `midi.rs` (with `_ => Screen::Pads`
as the fallback), the stale comment on `Shared::screen` ("0 pads, 1 settings, 2 files, 3 chop"),
keymap section names, help groups, learn groups, and `if screen == Screen::Fx` exceptions in
`Profile::resolve_in`. A new screen means edits in all of them. A missed one makes the
controller act as on the pads screen without an error.

## 2. Target design

### 2.1 One focus value

In `crates/input` add:

```rust
/// What the input does now. One value, published by the UI as one atomic store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Focus {
    pub screen: Screen,
    pub mode: Mode,
    pub bank: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Pads sound. Screen keys act.
    Normal,
    /// The edit of the selected object is open: the `[edit]` layer goes first; pads sound.
    Edit,
    /// Marking is on: a pad hit marks or unmarks the pad and the pad sounds.
    Marking,
    /// A file or a pad is in hand: the next pad hit is the target; pads are silent.
    InHand,
    /// The chop screen: pad hits go to the chop logic (listen or assign).
    Chop,
    /// A question is open: only confirm and back act; pads are silent.
    Ask,
    /// A list picker (folder, project) is open: list keys, confirm and back act.
    Picker,
    /// A text line is open: keys type; pads are silent.
    Text,
    /// Learn on the screen: controls go to the learner; nothing acts.
    Learn,
}
```

Add `Focus::pack(self) -> u32` and `Focus::unpack(u32) -> Option<Focus>`. `midi::Shared` gets
one `focus: AtomicU32` in place of `bank`, `screen`, `hand`, `edit`, `learn`. Later modes
(`Panel`, `Step`, `Erase`, `SlotPick`) are added in their own changes. Recording is a
transport state, not an exclusive pad mode: pads still sound while recording.

`App::focus(&self) -> Focus` is the **only** place that computes the mode from the `App` fields.
It holds the priority order that the guard chains hold now, written once, with a comment per
line. Order (first match wins): `Text` (pad naming, browser filter, a typed project name),
`Picker`, `Ask`, `Learn` (screen learn, MIDI run, settings learning), `InHand`, `Chop` (screen
is Chop), `Marking`, `Edit`, `Normal`. Check this order against the current code before you
write it. Where the current code has a different priority on keyboard and MIDI, that is a
divergence: list it in the design of change 85.

### 2.2 One route for pad hits

In `crates/input`:

```rust
pub enum PadRoute {
    /// Sound it now (fast path) and tell the UI.
    Sound,
    /// Do not sound; the UI decides (target, chop, mark without sound, ...).
    Ui,
    /// Do nothing.
    Ignore,
}

pub fn route_pad(focus: Focus) -> PadRoute;
```

A pure function, a table, unit tests for every `Mode`. Both callers use it:

- the MIDI thread: `Sound` → push `Command::PadPress` to the engine as now, then send
  `Event::PadHit { accepted }`; `Ui` → send `Event::PadHit { accepted: false }`; `Ignore` →
  only the input log line.
- `App::run` for keyboard pads: the same call, the same three branches.

After the route, the UI handles a pad in **one** function `App::pad_hit(abs, velocity, sounded)`
for both sources. `midi_event`'s `PadHit` arm and `apply`'s `PadPress` arms call it. Delete the
duplicated guards.

### 2.3 The stale snapshot race

The UI publishes `focus` once per frame. A controller button can change the mode (for example
"take file"), and a pad hit can arrive before the UI has handled the button. Then the MIDI
thread routes the pad with the old focus.

Fix: `Shared` gets `ack: AtomicU64`. The MIDI thread numbers each `Event::Action` it sends
(`seq`). The UI stores the `seq` of the last action it handled into `ack`, after it publishes
the new focus. While `sent_seq > ack`, the MIDI thread routes every pad hit as `Ui`. The cost is
UI latency only in the short window after a controller command.

### 2.4 Steps for knobs

In `crates/input/src/midi.rs`:

```rust
pub enum Resolved {
    Action(Action),
    Param { control: String, param: Param, value: f32, takeover: bool }, // absolute, as now
    Step { control: String, target: StepTarget, steps: i32 },           // relative, new
}

pub enum StepTarget {
    Param(Param),
    /// A list or a value that has only steps: files, projects, settings rows, banks, pages.
    List(ListTarget),
}
```

Profile syntax for a relative encoder:

```toml
e1 = { cc = 30, type = "relative", encoding = "offset64" }
```

Encodings: `offset64` (64 + n, so 65 = +1 and 63 = −1; the SMC-PAD sends this, measured
elsewhere), `twos` (1..63 = +, 127..65 = −), `signbit` (1..63 = +, 65..127 = −(n − 64)).
Unknown encoding: refuse the profile with a clear message.

Each `Param` gets a step rule in the app: `fn step(&mut self, param, steps, fine)`. Coarse and
fine steps per parameter are a table in the design of change 90 (for example: pad volume 1 dB /
0.25 dB, pitch 1 semitone / 10 cents only if cents exist, cursor 1 % of the view / 1 pixel
column, threshold 1 dB). Acceleration: the app counts steps of one control inside a 40 ms window;
the multiplier is 1, 2, 4, 8 by speed. `fine` (a held modifier) turns acceleration off.

Takeover does not apply to `Step`.

### 2.5 Tap and hold for buttons

The MIDI thread gets a `Gestures` tracker per (port, channel, control). Profile syntax:

```toml
[screens.global]
stop = { tap = "ui.back", hold = "layer.panel" }
```

A plain string binding (`b1 = "bank.prev"`) keeps today's behavior: act on press. A binding
with `hold` acts like this:

- press: start a timer; nothing acts yet;
- another control is used while the button is down: this is a hold at once;
- the button is down for `HOLD_MS` (300 ms, a setting): a hold;
- release before a hold: the `tap` action;
- a hold: the `hold` action gets `down` at the hold moment and `up` at the release.

A hold action is either a **layer** (`layer.panel`, later `layer.erase`, `layer.repeat`) or a
plain action with a down/up meaning (`record.mic`, on Alt+R after 83a). A layer is part of `Focus` (change 92 adds
`Mode::Panel`); the MIDI thread knows it at once, because it sees the button itself. It also
sends the layer to the UI, so the screen can draw it.

Only buttons with a `hold` binding wait for the release. Say this in the spec: such a tap acts
on release, not on press.

### 2.6 Confirm

Add `Action::Confirm` (`ui.confirm`). With `Mode::Ask` or `Mode::Picker`, the keymap and the
profiles resolve the confirm key to `Confirm` before the screen keys. Keyboard default: `Enter`.
All `Ask` variants and the pickers answer `Confirm`. The old answers (`ChopCommit`, `BrowserHold`
and the others) keep working, so existing learned bindings do not break. Add it to the GLOBAL
learn group.

### 2.7 One screen registry

In `crates/input`:

```rust
impl Screen {
    pub const ALL: [Screen; N];
    pub fn code(self) -> u8;
    pub fn from_code(code: u8) -> Option<Screen>;
    pub fn key(self) -> &'static str;   // "pads", "files", ... keymap and profile section names
    pub fn title(self) -> &'static str; // help and learn group titles
    pub fn rules(self) -> ScreenRules;  // e.g. FX: no edit layer, inherits page controls
}
```

Every place in §1.6 uses it. `from_code` failure is a logged error and the event is ignored; it
never falls back to Pads. Replace the `if screen == Screen::Fx` code in `Profile::resolve_in`
with `ScreenRules`. A unit test walks `Screen::ALL` and checks that each screen has a keymap
section, a help group and a learn group.

## 3. The changes

Each change: a branch, OpenSpec artifacts first, then code. "No behavior change" means all
current scenario tests stay green with no edits to them.

### Change 84 — `84-input-parity`

Goal: a test harness that drives the same scenario from the keyboard and from raw MIDI bytes,
and the first fix it finds (the chop pad divergence).

Tasks:

1. Test helper in `crates/app` (test module or `tests/common`): `Drive` with
   `key(code, mods)`, `release(code)`, `midi(port, bytes)` and `frame()` (runs one UI frame:
   publish shared state, drain MIDI events). The MIDI path must go through the real code: byte
   parse → `Profile::resolve_in` → the MIDI thread routing function → `App::midi_event`. If the
   routing is inside the thread loop now, move the body into a function that the loop and the
   test both call. This move is not a behavior change.
2. Two test profiles: the shipped X-Session profile, and a plain one with pads on notes 36–51 and
   buttons on CC 25–29 (close to the SMC-PAD).
3. A macro or a function `both(|d| { ... })` that runs a scenario once with keys and once with
   MIDI, and compares the commands sent to the engine and the visible state (selected pad, plan,
   in hand, marks, flash text).
4. Parity scenarios: pad hit on Pads; pad hit with Edit open; marking; file in hand → pad;
   `Ctrl+X` take → pad (swap); question open → pad does nothing; learn open → pad silent;
   Chop listen with no linked pads; Chop listen with linked pads; Chop assign by hand.
5. The chop scenarios fail with MIDI now. Spec delta in `input-devices`:
   "Requirement: A controller pad acts as a key pad. The system SHALL handle a pad hit from a
   controller in the same way as the same pad key, on every screen and in every mode." Fix:
   in `App::midi_event` send chop-screen hits to `App::chop_pad`. Carry the original hit
   velocity in `Event::PadHit`, which does not contain it yet. Keep the existing no-fast-sound
   route on Chop; do not trigger twice or replace the velocity with a constant.
6. Write every other divergence you find into `design.md`, "Found divergences". Fix only the
   chop one here. The others go into 85 as decisions for the owner, or into their own change.

Out of scope: any change to keys, profiles or the screen.

How to verify (owner): on the chop screen press P, hit pads on a pad controller with the
default profile: the plan fills as with the keys. The X-Session Pro has no pads; use it for
its existing knobs and safe buttons alongside the pad controller.

### Change 85 — `85-input-modes`

Goal: `Focus`, `Mode`, `route_pad`, one `App::pad_hit`. The code moves keep behavior.
The `ack` race correction is a separate, explicit behavior fix within this change: specify
and test it before code. Do not describe the whole change as behavior-neutral.

Tasks:

1. `Focus`, `Mode`, `pack` / `unpack` with unit tests (round trip for all values).
2. `App::focus()` with the priority list; unit test per mode from a built `App` state.
3. `route_pad` table with a unit test per mode.
4. `Shared.focus` and `Shared.ack`; delete the five old atomics. The MIDI thread reads one
   value per message.
5. The MIDI routing function uses `route_pad` and the `ack` rule.
6. `App::pad_hit` for both sources; delete the duplicate guards in `apply` and `midi_event`.
7. The status line or the help shows the mode name in debug builds only (help for the agent's
   UX runs; not for the owner).
8. Move the dispatch (`run`, `apply`, `midi_event`, `pad_hit`, `focus`) into
   `app/src/dispatch.rs`, as its own commit with only the move (`CROSS-CUTTING.md` §5).
9. All parity scenarios of 84 stay green. Add: "a controller button takes a file and a pad
   arrives in the same frame: the pad is a target, not a sound" (the race).

Out of scope: new modes, new keys.

### Change 86 — `86-screen-registry`

Goal: §2.7. No behavior change.

Tasks: the `Screen` methods; replace the code tables in `main.rs` and `midi.rs`; `ScreenRules`
for FX; the registry unit test; fix the stale comment. Test: an unknown code is ignored and
logged, the pads screen does not act.

### Change 87 — `87-status-badges`

The visual base: badges in the title line from `App::focus()`, pad glyphs and the grid tint for
the existing modes, the message model with levels (in place of `App.flash`), the value pop-up,
the status line segments, the question answer line. It needs `Focus` (85). The full plan is in
`VISUAL.md` §3. It comes before 88, so that `ui.confirm`, the button hold and the encoders show
their state from the start.

### Change 88 — `88-ui-confirm`

Goal: §2.6.

Spec deltas: `keyboard` (Enter is confirm in a question and in a picker), `midi-learn`
(`ui.confirm` in GLOBAL), `screen-help` (the question line names the confirm control of the
connected controller, if bound). Scenarios: every `Ask` kind answered by `ui.confirm` from the
keyboard and from a learned MIDI button (parity harness). Old answers still work.

### Change 89 — `89-button-hold`

Goal: §2.5 for MIDI buttons.

Spec deltas: `input-devices` (tap / hold bindings, hold time, "another control makes a hold",
taps on such buttons act on release), `midi-learn` (learning a tap and a hold on one button: the
learn line asks "tap or hold?" with the two existing value keys; default tap), `audio-settings`
or settings tab MIDI (the hold time, 200–600 ms, default 300).

Tasks:

1. `Gestures` tracker, pure, tested with timed event lists (no sleeps: pass `Instant`s).
2. Profile syntax `{ tap, hold }`; old plain strings unchanged.
3. Learned maps store the tap and the hold action of a control (config migration: old entries
   are taps).
4. Hold of `record.mic`: down starts, up stops (same as `Alt+R` after 83a).
5. Disconnect and lost releases clear all gesture state of the port (`CROSS-CUTTING.md` §2.2);
   tests with recorded event lists.
6. No layer yet (layers come in 92). A `hold` binding to an unknown layer is refused with a
   message.

### Change 90 — `90-relative-encoders`

Goal: §2.4.

Spec deltas: `input-devices` ("Relative encoders": encodings, acceleration, fine step with a
held modifier, no takeover), `editing` (each parameter's coarse and fine step: one table),
`sample-loading` (a knob scrolls the file list), `projects` (a knob scrolls the project
picker), `audio-settings` (a knob moves the rows).

Tasks:

1. `Resolved::Step`, `StepTarget`, the encoding parser and decoders with byte tests for each
   encoding, including the SMC-PAD `63 / 65` bytes.
2. `App::step(...)` for every `Param`, from the steps table. Absolute knobs keep their current
   code path.
3. List targets: `files.scroll`, `projects.scroll`, `settings.row`, `edit.page`, `bank`.
4. Acceleration and fine step, tested with timed event lists.
5. Parity: the same parameter moved by keys (J/L) and by a relative knob gives the same value
   for the same number of steps (without acceleration).

How to verify (owner): after 91 sets the SMC-PAD to relative, turn a knob slowly: the value moves
in small even steps; turn fast: big moves; the value never stops at an end of the knob.
