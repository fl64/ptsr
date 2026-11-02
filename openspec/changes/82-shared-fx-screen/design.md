## Context

FX 1 and FX 2 are shared by the project. Their settings currently appear as pages in the
pad edit. Pad drive and the two sends belong to the pad. The project already saves both
slots and their effect settings.

## Goals / Non-Goals

Separate shared effects from pad parameters without changing their sound or project
format. Do not add effect types or slots.

## Decisions

F4 opens the FX screen. FX 1 and FX 2 are distinct blocks. Tab selects the next block;
Shift+Tab selects the previous block. The selected block is visibly marked and its
parameter names and values are visible. I/K select a parameter and J/L change it. The
same three learned page knobs change type, first parameter and second parameter of the
selected block. Pickup, jump and scale follow the existing per-device takeover setting;
changing the selected block cannot use the previous block's pickup state.

Pads remain playable from the keyboard and MIDI. The pad edit keeps drive and both sends,
but no longer offers pages for shared FX. Existing saved projects and learned MIDI
bindings must keep their meaning. `Ctrl+Z` undoes a shared effect change. F1 and the
bottom help name the screen controls and which block is selected.

The FX controls use two compact cards. Each card has only its title and three parameter
rows. Cards never expand to fill the terminal. A wide terminal shows the cards beside
each other. A narrow terminal stacks them. Labels, values and knob numbers stay visible.

Old page assignments use the pad edit layer as their source. A replacement assignment
must suppress the previous control. Clearing an FX assignment must not restore an
inherited control. Migration uses the existing MIDI group map format, not a new format.
A physical MIDI release must stop a previously held pad even if the screen mapping changes.

### X-Session Pro replacement profile

Use the factory map from Appendix A of the manufacturer guide:
https://www.manualsdir.com/manuals/122289/m-audio-xsession-pro.html?page=9
Cross-check: https://raw.githubusercontent.com/mixxxdj/mixxx/main/res/controllers/M-Audio_Xsession_pro.midi.xml

Keep the profile name, MIDI numbers and numeric control identifiers. The physical labels
and complete action table belong in the user stories, not guessed capture-order labels.

| Physical control | MIDI | Default role |
|---|---|---|
| Left / right Headphone | Note 44 / 45 | Files / FX screen |
| Left backward / forward | Note 46 / 43 | Pads: select pad; Files: select file; Chop: select slice; edit: previous / next page; FX: previous / next block |
| Right backward / forward | Note 56 / 57 | Previous / next bank |
| Left Cue | Note 58 | Pads: edit; Files: file in hand; Chop: cut; edit: close |
| Right Cue | Note 59 | Back; Files: open Chop; Chop: slice edit; edit: close |
| Left / right Play | Note 70 / 69 | Files: preview toggle; Chop: play / stop; other screens: unassigned |
| Left numbered knobs 1–3 | CC24–26 | Edit / FX: page parameters; Chop: start, end, cursor; Pads: knob 1 volume, others unassigned |
| Left Hi / Mid | CC27 / 28 | Chop: region start / end |
| Right Mid / Low | CC35 / 36 | Chop: threshold / zoom |
| Left Volume / Pitch | CC11 / 12 | Selected pad volume / pitch on all screens |
| Right Volume | CC14 | Master volume on all screens |

Right Pitch CC15, crossfader CC17 and its inverse CC20 stay unassigned. No default button
deletes a cut, applies a layout or confirms replacement. These actions remain available
from the keyboard and MIDI learn. Both Play buttons keep preview roles inside edit mode.
Use dedicated safe actions `pad.edit_only`, `chop.edit_only` and `browser.take_only`
for Cue. They open edit or take a file without applying a pending plan, placing a held
file or answering a confirmation. Existing Enter and learned actions keep their behavior.
FX inherits page controls from the edit layer. Do not duplicate global controls in FX:
global learning and clearing must affect all screens. Preserve inherited user page knobs
on FX during first initialization and migration, including configurations without a seeded marker.
A button release never repeats the action. Profile initialization must keep multiple
factory controls for one action (both Play and Cue buttons). An assignment that existed
before initialization still blocks competing defaults for its action or control. Explicit
MIDI learn keeps its existing one-control-per-action replacement behavior.
New defaults apply only on first setup or a
confirmed Reset to profile. Existing seeded and learned maps, including old Play roles,
remain unchanged until that reset. No user config format or audio algorithm changes.

## Risks / Trade-offs

Existing bindings may refer to the former edit layer. Preserve their function on the FX
screen and test both old bindings and new page controls. Switching the selected block
must not change an effect or sound by itself. Check small terminal sizes and both
keyboard layouts.

## Open Questions

None.
