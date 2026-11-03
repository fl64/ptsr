## Context

Source plan: `docs/plans/KEYMAP-CLEANUP.md`. Baseline: preparation commit `0b1da2c` on
`plan/controller-first-patterns`. No feature code changes are included in this step.
The owner confirmed option A and continuation in this same branch on 2026-11-03.
Only 83a is active. Keep 84 deferred until the keymap cleanup is accepted.

Existing code gives the implementation boundaries:

- `config/keymap.toml` binds Ctrl+R globally, Ctrl+N and Ctrl+Y on Chop, and J/L to banks
  on Pads. The edit layer already has value keys.
- `crates/input/src/lib.rs:1656` holds the keymap and overlays. At line 1910 the chord
  parser handles Ctrl and Alt; Shift+Tab is special, but Shift+H is not supported.
  `resolve_in` at line 1752 resolves events and microphone release.
- `crates/app/src/main.rs:1694` handles assign order through `ParamNext`; keep this
  compatibility. `PadTake` and `pad_in_hand` handle swap, not copy. `Ask::PutFile`
  shows the existing replacement question. Copy needs its own saved pad state, not
  only a path, because it includes settings, source links and a custom name.
- App loads the owner's keymap at line 6145 and resolves keys at line 6032. It already
  has startup notices. Help and learn use the action catalogue and resolved bindings.
- `crates/app/src/chop.rs:485` steps hit thresholds down. Reverse stepping must use
  the same cuts, threshold and undo history, not a second detection rule.
- Main-spec coverage is checked by `crates/app/tests/scenarios.rs`. Keep the deltas
  active during implementation; add their named tests before archive.

## Goals / Non-Goals

Make the shipped keys consistent, retain owner controls and add the three planned
logical actions. Preserve pad safety, microphone behavior and audio timing. No audio
callback change, dependency, configuration rewrite, transport or later refactor.

## Decisions

### Confirmed keymap

| Context | Key | Logical action |
|---|---|---|
| Global | Alt+R | `record.mic`: hold with releases, toggle without them |
| Global | Ctrl+T | `record.output`, unchanged |
| Chop | Ctrl+G | `chop.normalize` |
| Chop | H / Shift+H | `chop.hits` / `chop.hits_back` |
| Chop, assigning | Tab | `chop.assign_order` |
| Pads, edit open or closed | J/L and Left/Right | `edit.value_dec` / `edit.value_inc` |
| Pads, edit open or closed | U/O | `bank.prev` / `bank.next` |
| Pads | Ctrl+C | `pad.copy` |

Ctrl+R stays free for pattern recording in 97; Ctrl+Y and Ctrl+. stay free for redo
and panic in 93a. Ctrl+B remains the Files favorite. Ctrl+D has the duplicate/double
role when a later change binds it. Do not add Ctrl+Shift+letter defaults.

Plain letters have screen roles: H can mean pad mode on Pads and hits on Chop. The
one-meaning test applies to Ctrl/Alt chords, not every plain letter. It compares the
last word of the action name, or a small allowlist that explains each exception. Check
edit layers too; owner overrides are not forced to obey the shipped-map test.

### Shift and legacy input

Add Shift distinction only as needed for explicit command chords, keeping Shift pad
velocity and current unbound-Shift fallback. A shipped Shift+H must be distinct from H
in both extended events and legacy uppercase text, including the Russian physical H
key. Shift+Tab and existing Ctrl/Alt bindings stay compatible. Where Shift+Tab has no
previous-page binding, keep its current Tab fallback rather than adding a new Settings
navigation action in this cleanup. Ctrl+C is a command in raw mode, not quit. Alt+R without releases toggles recording; tests drive both event
forms. A remap of old Ctrl+R to microphone retains the previous behavior.

### Notices respect input priority

Use the current flash and startup notice flow, not change 87's new notice model. Put
the moved-key table in App, not the shipped keymap. Resolve text input, questions,
learning and actual key bindings before considering a moved-key flash. A note must not
execute the former action or intercept a valid user or shipped action.

Rows: global Ctrl+R -> Alt+R; Chop Ctrl+N -> Ctrl+G and Ctrl+Y -> H; Pads J/L -> U/O.
The plan's notice condition is important: J/L have new default value bindings, so those
rows do not show a moved-bank notice in the default map. Test that values change and
no bank notice appears. Only unbound old keys show their move. An explicit owner
entry, including `none`, suppresses the moved-key notice for that entry.

At start, report each conflicting changed default once per chord and screen. Include
new chords and changed existing chords, including Shift+H and the Pads value keys.
An owner entry for the same action is not a conflict. Owner bindings and explicit
removals win; do not rewrite their file. Deliver notices one at a time, as the current
startup flow does. Remove the Ctrl+R moved-key row when 97 binds it; remove the rest in
the first change after version 0.2. This lifetime is already in `docs/BACKLOG.md`.

### Pad copy is a saved pad, not a swap

Proposed default: take a snapshot when Ctrl+C or a learned `pad.copy` acts on the
selected loaded pad. Reuse the existing pad snapshot/restore and engine command
helpers. Share sample data with `Arc`; retain its file, own bounds or slice link,
mode, volume, pitch, envelope, filter, drive, mute group and sends. Set the new custom
name to the displayed source name plus ` 2`. Do not copy held state, transient hit
lights, selection or marks. Do not change the source or allocate sample data again.

Use a distinct copy-in-hand state and replacement question. Publish the current
in-hand flag so MIDI targets go to App, as file-in-hand and swap targets do today.
Target hits do not also trigger sound. Keep original releases, epoch filtering and
queue-acceptance behavior; do not introduce 85's mode model or acknowledgement fix.
Copy takes target priority over marking, without clearing the owner's marks.

An occupied target asks by name. Enter applies only to the target the question names;
if its state changed, ask again. Esc closes the question, keeps the target and retains
the copy in hand for another target. Esc outside the question or a hit on the original
source drops the copy with no undo step. A successful placement clears the hand and
makes one undo step for the target only. It is saved through existing project/session
mechanisms. Copying onto a loading target must not race a pending file load.

### Chop command compatibility

`chop.assign_order` switches by-hand/in-order only while a plan is open. It does
nothing in listen or slice edit. Keep the existing `edit.param_next` and
`chop.auto_mode_next` assignment behavior for owner maps and learned controls. Assign
mode still ignores cuts and threshold changes; neither new action applies a plan.

H keeps the downward cycle -6, -12, -18, -24, -30, -6 dB. Shift+H goes to the previous
higher step and stops at -6 dB; it does not wrap. A continuous knob threshold goes to
the next higher step of that list while below -6. A knob value already at or above
-6 stays unchanged on reverse; moving it down to -6 would add hits, not remove them.
Both keys set the shared hit rule, replace cuts with the current link safeguards and
make one undo step when cuts or threshold change. A clamped reverse is a no-op with
the threshold named in the flash.

### Preserve existing scenario names

A MODIFIED block replaces a whole requirement. Carry every current scenario forward.
Two names mention old keys: `Ctrl+Y cuts at the marks` and `L turns to the next bank`.
Keep them as explicit owner-remap compatibility scenarios and add separate default H
and U/O scenarios. Do not claim that the old shipped keys still perform those actions.
Rename `Banks on J and L` to `Keyboard bank controls` with a RENAMED delta, then modify
the new header. No unrelated requirement is dropped or synchronized in this step.

## Risks / Trade-offs

- This is more than a TOML edit: Shift resolution, learn, questions, undo and startup
  notices cross input paths. Keep commits small and run `task check` after each task.
- A bare-sample copy test misses settings, link, name and sound differences. Test full
  state, shared sample identity, persistence and an offline source/copy comparison.
- Main specs still describe the shipped program until archive. Active deltas describe
  the next behavior; current tests do not prove that behavior is implemented.

## Open questions

These are explicit defaults for the implementation proposal, not extra owner approvals:

- Snapshot at pickup or at placement? Default: at pickup, so changes to the source
  while the copy is in hand do not silently change the pending copy.
- Copy an empty, missing or loading source? Default: refuse with a plain notice and
  leave pads and existing in-hand state unchanged. Refuse a loading target until ready.
- What if an owner explicitly disables an old chord? Default: `none` stays silent;
  do not replace their choice with a moved-key flash.
- A hits-back press before hit chopping, or after a knob value? Default: use the next
  higher listed step below -6 dB; an explicit value at or above -6 stays. With no
  explicit threshold, start at -6 dB. The shared initial -12 dB rule stays until used.
- Must cleanup add reverse tab navigation on screens that never had it? Default: no;
  keep existing unbound Shift+Tab fallback. The three new planned action names are
  sufficient; a separate Settings navigation action is not hidden in this change.
