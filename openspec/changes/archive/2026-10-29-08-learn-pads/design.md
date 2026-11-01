# Design — 08

## Where the actions live

16 actions `pad.1` … `pad.16` in `ACTIONS`, `Group::Global`. Pad hits must answer on every screen — only GLOBAL acts everywhere (a screen group would silence the instrument outside its screen). They sit among the other global rows of the MIDI tab, which already scrolls and jumps by group.

`Action::parse` learns the pattern `pad.N` (1..=16) → `PadPress { pad: n-1, velocity: 1.0 }`. Names reuse the existing `bank.N` style, so profiles and the user config need no format change.

## Velocity and release in the MIDI dispatch

Today a profile `Control::Pad` carries velocity, but a learned `Control::Button` throws it away (`Action::parse` has no velocity). Two small changes in `midi.rs`:

- **Press**: after parsing the action of a button press, if it is `PadPress`, replace the velocity with `value / 127.0` (a note's velocity, a CC button's value).
- **Release**: `Control::Button` currently ignores releases. If the control's action is a pad (`pad.N`), a release sends `PadRelease { pad }` — that is what ends a gate pad without a click. Other actions keep ignoring releases, as now.

Knobs: a knob action only ever collects CC turns; `pad.N` is a press action, so learn takes the first note or CC press for it — the same rule every button action follows (raw MIDI does not tell a knob from a button).

## App side

No new wiring: the app already turns `Action::PadPress` / `PadRelease` from MIDI into engine commands for profile pads; learned controls go through the same `Resolved::Action` path. The MIDI tab rows come straight from `ACTIONS` labels.

## Open questions

1. **Group.** Default taken: GLOBAL (works on every screen), 16 more rows in the longest group. Alternative: a separate "PAD HITS" group — the MIDI tab groups are visual only, it would act like GLOBAL anyway; skipped as extra chrome.
2. **CC release.** Default: CC value 0 counts as release (as buttons already do). A control that never sends 0 never ends a gate pad — same behaviour as controller buttons today.
