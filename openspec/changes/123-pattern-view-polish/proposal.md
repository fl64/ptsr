# Proposal

## Why

The A1 overview spends most of its rows on pads without hits. Its step widths vary, so step numbers can be hard to follow. All hits have one brightness, and the step editor can only remove hits.

## What Changes

- Show pads with visible hits and the selected pad by default. `Alt+H` shows all rows.
- Give each step a fixed three-cell width. Align each step number with its cell and keep straight and triplet beat groups clear.
- Show soft, medium, and loud hits with three brightness levels. Keep off-grid and multiple-hit marks. Keep the live playhead.
- Add an empty step with a silent keyboard or MIDI pad hit. Keep the incoming velocity. A hit on an occupied step removes one identified event under the existing choice rule. Give each accepted edit one Undo step.
- Mark the selected time step across all pad rows. Show the pressed pad as the edit target. A pad press changes only that pad in the selected step. Hits on other pads stay in place.
- Name Undo in edit feedback. Show its effective key in the help band.

The change SHALL keep saved format 5, logical action IDs, the sound engine, and ordinary pad playback. It SHALL add no dependency.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `patterns`: A1 overview rows, grid, velocity feedback, and step editing.
- `screen-help`: the visible meaning of the existing overview toggle and step editor.

## Impact

The change affects the A1 projection, terminal drawing, and the existing silent edit path. It uses the current ordered publication and Undo path. It does not change the audio callback or project fields.

## Out of scope

The bank mixer, new pattern slots, and new sound effects remain separate changes.

## How to check

### Keyboard

1. Open F5 with hits in two banks. Check that the overview shows only rows with hits plus the selected pad. Press Alt+H to show all rows and press it again to return.
2. At 80x26 and 120x40, check step numbers over fixed-width cells in every view division. Check straight, triplet, off-grid, multiple-hit, and moving-playhead states.
3. Open F7. Move to an empty step with J/L. Check that the time column crosses every visible pad row. Press a pad. Its row becomes the target and gets a hit without immediate pad sound. Press the same pad again to remove its hit. Use Ctrl+Z after each edit.
4. At a step with a hit on pad 1, press Q. Check that both hits remain. Press 1 to remove only its hit. Check that the result names the bank, pad, and step.
5. Use Shift with a keyboard pad. Check full velocity. Save and reopen a disposable project to check the edited A1.

### Controller

1. Learn `edit.value_dec` and `edit.value_inc` on two controller controls. Move the time column one step in each direction. Use a raw MIDI pad at low, medium, and high velocity. Check the same cells and silent edit path.
2. Release a Gate pad and leave step edit. Check that the original held sound and later ordinary pad play stay unchanged.

### Views and sound

Review the local `renders/screens/pads-pattern-overview-velocity-80x26.ansi`, `pads-pattern-overview-velocity-120x40.ansi`, `pads-pattern-step-add-80x26.ansi`, and `pads-pattern-step-add-120x40.ansi`. Add and review `pads-pattern-step-target-80x26.ansi` and `pads-pattern-step-target-120x40.ansi`. Compare the existing overview and step frames at both sizes. Listen to `renders/97-pattern-record.wav` and the new `renders/123-step-add.wav`. This change adds no effect.
