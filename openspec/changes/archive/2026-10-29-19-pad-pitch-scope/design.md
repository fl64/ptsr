# Design — 19

## Pitch in the engine

A voice reads the sample at a rate `2^(semitones/12)` with linear interpolation (a fractional read position per voice, no allocation). A change on a sounding voice glides over ~20 ms, so there is no click and no zipper. The region end is reached sooner at a higher pitch; the end fade stays as it is.

## State

`Pads` grows `pitch: [i8; PADS]` beside `volume_db`. The scope is UI state, not saved: `Scope::Pad | Bank | All`, plus `marked: BTreeSet<usize>` and `marking: bool`. The pads of the scope: marked pads when there are marks and the scope is `Pad`; otherwise the selected pad, the 16 pads of the current bank, or all 128.

## Keys (pads screen)

| Key | Action | Note |
| --- | --- | --- |
| `Tab` | `pad.param_next` | volume → pitch → volume |
| `-` `=` | `pad.param_dec` / `pad.param_inc` | the picked parameter, for the scope; replaces `pad.volume_down/up` |
| `Enter` | `pad.mode_next` | for the scope |
| `Ctrl+A` | `pad.scope_next` | pad (or marked) → bank → all |
| `M` | `pad.mark_toggle` | marking on / off |
| `Esc` | `ui.back` | drops the marks first, then goes back as now |

All are on the right half or Ctrl+letter; the pad block and Shift stay free. `Backspace` (clear) keeps acting on the selected pad only: losing samples on a whole bank by one key is not a parameter change.

## Knobs

`pad.volume` (exists) and `pad.pitch` (new) take over against the selected pad; the delta of a turn goes to every pad of the scope.

## Open questions

- Cents (fine pitch): left out; semitones first. Default: no cents.
- Undo of pad parameters: left out; a relative change is undone by the opposite keys, except at a limit. Default: no undo, the pop-up says how many pads hit the limit.
- Pads screen knob for the parameter picked with `Tab` (one knob for whatever is picked): left out. Default: one knob per parameter.
