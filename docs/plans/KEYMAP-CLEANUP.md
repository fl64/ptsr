# Plan: keymap cleanup (change 83a, before 84)

Read `README.md` first. This change makes the shipped keys consistent before the input refactor
and before patterns add new keys. It is small, but it changes keys that the owner already uses,
so it must tell the owner about every moved key.

## 1. Why now

- Muscle memory is still young. Each later change adds keys (patterns, panic, redo); a clean base
  costs less now than later.
- The plans need `Ctrl+R` (pattern recording, change 97) and `Ctrl+Y` (redo, change 93a). Both
  are taken now by other actions.
- Change 84 (parity tests) takes the keymap as the reference. The reference must be right first.

## 2. Rules that the keymap must follow after this change

Write these rules into the `keyboard` spec as one requirement ("Key meaning is consistent"), and
add a unit test that checks the shipped keymap against them where a machine can check it.

1. The left half (`1234 QWER ASDF ZXCV`) is pads only, with and without Shift. (Exists.)
2. **One chord, one meaning.** A Ctrl or Alt chord means the same action, or the same kind of
   action, on every screen where it is bound. Test: for each Ctrl / Alt chord bound on more than
   one screen, the action names share the same last word (`*.normalize`, `*.rename`) or are on an
   allowed list in the test with a reason.
3. **The same role keys on every screen:** `I` / `K` up / down (parameter, row, zoom), `Enter` the
   main action of the screen, `Esc` back, `Backspace` delete the selected thing, `M` put a mark,
   `Tab` / `Shift+Tab` the next / previous page or group.
4. **On screens with a parameter panel, `J` / `L` change the value of the selected parameter**
   (owner decision, default yes; see §4.3).
5. **Recording is on R:** `Ctrl+R` records what you play (patterns), `Alt+R` records the
   microphone, `Ctrl+T` records the output (resampling).
6. No `Ctrl+Shift+letter` chords: terminals take them (kitty: `Ctrl+Shift+R` resizes the window).
7. Every new chord works without the kitty keyboard protocol, or the spec says what replaces it
   there (spec "no kitty protocol keeps the Ctrl combos").

## 3. The key changes

| # | Screen | Key | Before | After |
|---|---|---|---|---|
| 1 | global | `Ctrl+R` | `record.mic` (hold) | free until change 97, then `pattern.record`. A press before 97 shows the moved-key notice for the microphone |
| 2 | global | `Alt+R` | — | `record.mic` (hold: record, release: stop), as `Ctrl+R` did |
| 3 | global | `Ctrl+T` | `record.output` | no change |
| 4 | chop | `Ctrl+N` | `chop.normalize` | free on chop (on pads it stays `pad.rename`) |
| 5 | chop | `Ctrl+G` | — | `chop.normalize` |
| 6 | chop | `Ctrl+Y` | `chop.hits` | free; change 93a binds `edit.redo` to `Ctrl+Y` globally |
| 7 | chop | `H` | — | `chop.hits` (one threshold step lower per press). `Y` (parts) and `H` (hits) are one column: two ways to chop |
| 8 | chop | `Shift+H` | — | `chop.hits_back`: one threshold step higher (new action; fine step rule of Shift) |
| 9 | chop | `Tab` | `edit.param_next` (in assign mode: in order / by hand) | `chop.assign_order` (new action, same behavior). `edit.param_next` keeps this behavior in assign mode, so old owner keymaps and learned MIDI bindings still work |
| 10 | pads | `J` / `L`, `Left` / `Right` | `bank.prev` / `bank.next` | `edit.value_dec` / `edit.value_inc` for the selected parameter, also with the edit closed (as on the FX screen) — **owner decision, §4.3** |
| 11 | pads | `U` / `O` | — | `bank.prev` / `bank.next` (the row above `J` / `L`, the right hand stays in place) — only with #10 |
| 12 | pads | `Ctrl+C` | — | `pad.copy`: the next pad hit gets a copy of the selected pad (§4.4) |
| 13 | global | `Space` | `metronome.click` | no change here; change 95 makes it `transport.play_stop` |

Not changed, but written down: `docs/VISION.md` had `Ctrl+B` for bake, while the keymap has
`Ctrl+B` for the favorite in Files. The favorite stays; bake gets another key when it comes
(VISION is updated). `Ctrl+D` means "duplicate or double" wherever it is bound.

Bank keys that stay: `PageUp` / `PageDown`, `Ctrl+1` … `Ctrl+8`, and LEFT / RIGHT of a controller.

Panic (`Ctrl+.`) and redo (`Ctrl+Y`) are bound in change 93a, not here. This change only frees
the keys and writes them into the keyboard spec as reserved.

## 4. Details

### 4.1 Moved-key notices

The owner presses an old key out of habit. Do not let it do nothing silently.

- A table in `crates/app` (not in the keymap): old chord, screen, the action that moved, its new
  key. Rows: `Ctrl+R` → `Alt+R` (microphone); `Ctrl+N` on chop → `Ctrl+G`; `Ctrl+Y` on chop →
  `H`; with #10: `J` / `L` on pads → `U` / `O` for banks.
- When the owner presses a chord from the table, and the chord is not bound in the owner's
  `keymap.toml` and not bound by the shipped keymap at that moment, show an Info notice:
  `· microphone moved to Alt+R`. Nothing else happens.
- When change 97 binds `Ctrl+R`, its row leaves the table.
- The table is removed in the first change after version 0.2 (write this in `docs/BACKLOG.md`).
- Before change 87 (the notice model) this uses the current flash.

### 4.2 The owner's keymap

The owner's `keymap.toml` lies over the shipped one (spec "The owner's keymap"). Rules:

- An owner binding wins, as now. If the owner bound `Ctrl+R` to `record.mic` himself, it stays.
- At start, if the owner's keymap binds a chord that this change gives to another shipped action
  on the same screen (`H`, `Ctrl+G`, `U`, `O`, `Ctrl+C`, `Alt+R`), show one Warn notice per chord:
  `! keymap.toml: H on chop hides chop.hits`. The owner's binding still wins.
- New action names (`chop.assign_order`, `chop.hits_back`, `pad.copy`) are learnable in F10 and in
  the MIDI tab, in their screen groups.

### 4.3 Owner decision: `J` / `L` on the pads screen (#10, #11)

Now `J` / `L` turn banks on Pads but change values on FX; `I` / `K` select a parameter on both.
So on Pads the owner selects a parameter with `I` / `K` and cannot change it with `J` / `L`
without `Enter` first.

- Option A (default): `J` / `L` change the value on Pads too; banks move to `U` / `O`.
- Option B: keep `J` / `L` as banks; then on FX values also need the edit open (rule 4 is
  written the other way).

Ask the owner before you start. Write the answer into the proposal. If the answer is B, drop
rows #10, #11 and the `J` / `L` row of §4.1, and write rule 4 for B.

### 4.4 `pad.copy`

- `Ctrl+C` on Pads: the selected pad is "in hand" as a copy. Badge / title: `COPY · hit a pad`
  (after 87: a PadMode badge). The next pad hit, in any bank, gets the copy: the sample reference,
  the slice bounds, the name with a ` 2` suffix, all pad parameters (volume, pitch, envelope,
  filter, mode, mute group, sends). The sample data is shared, not duplicated in memory.
- An occupied target asks first, like a file put on an occupied pad (`Ask::PutFile` style).
- `Esc` drops the copy. A pad hit on the same pad drops it too.
- One undo step.
- A controller pad hit is the target too (it goes through `App::pad_hit` after 85; before 85,
  follow the in-hand path that `pad.take` uses for the MIDI thread: set `hand`).
- In `raw mode` the terminal gives `Ctrl+C` as a key; check this in a test with the kitty protocol
  on and off. `Ctrl+Q` stays the only quit.

### 4.5 `chop.hits_back`

The threshold goes one step higher (fewer cuts). It stops at the start value of `chop.hits`.
The flash names the threshold, as `chop.hits` does.

## 5. Spec deltas

- `keyboard`: the requirement of §2; the table of reserved keys (`Ctrl+R` from 97, `Ctrl+Y` and
  `Ctrl+.` from 93a); the moved-key notices; the owner keymap warnings.
- `recording`: the microphone on `Alt+R` (hold); output on `Ctrl+T`.
- `chopping`: normalize on `Ctrl+G`; hits on `H`, back on `Shift+H`; `chop.assign_order` on `Tab`.
- `editing` or `pad-playback`: `J` / `L` on Pads (option A) and banks on `U` / `O`.
- `pad-assignment`: `pad.copy`.
- `screen-help`: the help groups and the hint texts with the new keys.
- `midi-learn`: the new actions in their groups.

## 6. Tasks

1. Ask the owner the question of §4.3. Write the answer into the proposal.
2. OpenSpec artifacts with the deltas of §5. Each moved key has a scenario: the new key acts; the
   old key shows the moved-key notice and acts as nothing else.
3. `config/keymap.toml`: the changes of §3, with comments in the existing style.
4. New actions in `crates/input` (`Action`, `ActionInfo`, names, learn groups): `chop.assign_order`,
   `chop.hits_back`, `pad.copy`.
5. `pad.copy` in the app (§4.4), with an undo step and the question for an occupied target.
6. The moved-key table and notices (§4.1); the owner keymap warnings (§4.2).
7. The consistency test of §2 (rule 2 and rule 6 by machine; the allowed list with reasons).
8. Help and hints: every text that names a moved key (search `hint_for`, the help groups, the
   flash texts for "Ctrl+R", "Ctrl+N", "Ctrl+Y", "J L").
9. Documents: `docs/USER-STORIES.md` (keys in the stories), `docs/LIVE-CHECKS.md`, `README.md` of
   the repository (key list), `docs/VISION.md` §17 keyboard line, `docs/plans/*` if a key there
   changes.
10. `task check` green; the X-Session tests unchanged (its profile binds actions, not keys).

## 7. How to verify (owner)

- `Alt+R` held records the microphone onto the selected pad; `Ctrl+R` shows "microphone moved to
  Alt+R".
- Chop: `Y` equal parts, `H` hits (each press finer), `Shift+H` back; `Ctrl+G` normalizes;
  `Ctrl+N` and `Ctrl+Y` show where their actions moved.
- Pads (option A): `I` / `K` pick a parameter, `J` / `L` change it; `U` / `O` turn the banks.
- Pads: `Ctrl+C`, hit another pad: a copy that plays the same; hit an occupied pad: a question.
- Help (F1) and the bottom line show the new keys only.
