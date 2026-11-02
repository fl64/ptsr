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

## Risks / Trade-offs

Existing bindings may refer to the former edit layer. Preserve their function on the FX
screen and test both old bindings and new page controls. Switching the selected block
must not change an effect or sound by itself. Check small terminal sizes and both
keyboard layouts.

## Open Questions

None.
