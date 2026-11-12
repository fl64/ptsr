# Proposal

## Why

The Pads strip shows the rhythm of one selected pad and a summary of its bank. The owner wants to see the rhythm of all pads together, as in a DAW. This change adds that overview before targeted step editing.

## What Changes

- F5 opens one scrollable A1 overview for all eight banks. Each bank section keeps 16 pad rows.
- A fixed bank index shows all eight banks. The current bank has a colored outline that matches its pad-screen color.
- All bank sections appear by default. Alt+H hides or shows empty sections. The current bank and every bank number stay visible.
- Each row uses the available width for its pad name and time cells. Empty pads keep their rows in a shown bank.
- The overview uses the existing eight view divisions, source-event marks and audio-driven playback position.
- Existing bank controls move through the bank sections. The view and playback keep their positions.
- Pad keys still play and select their pads. F2 or Esc returns to Pads.
- F1 and the help band explain the marks, view controls, bank controls and empty-bank switch. Other screens label F5 `pattern`. The screen appears in the offline catalogue.

## Capabilities

### New Capabilities

None. The overview extends the existing `patterns` capability.

### Modified Capabilities

- `patterns`: project-safe projection of visible A1 hits across all eight banks.
- `keyboard`: F5 opens the overview and preserves existing pad and view controls.
- `midi-learn`: a learnable overview action uses the same screen route.
- `screen-layout`: the scrollable bank sections, wide row names, playback plate and catalogue states.
- `screen-help`: effective controls and the meaning of event marks.

## Impact

The implementation affects App view data, screen routing and terminal drawing. It uses the existing pattern compiler and audio snapshot. It does not change recorded hits, sound, project files, dependencies or the audio callback.

## Out of scope

- Adding, moving or removing pattern hits in the overview.
- A grid that puts all 128 pad rows in one terminal viewport. Bank sections scroll through the viewport.
- A new SMC-PAD factory profile or hardware acceptance.
- The targeted removal editor in the deferred 98a draft.

## How to verify

### Keyboard

1. Use a disposable project. Record A1 with hits on several pads in banks 1 and 2.
2. Press F5. Verify the eight-bank index and the 16 rows of the current bank, including empty pads.
3. Start playback. Verify that the plate moves with A1 and the sounds do not change.
4. Press U or O to move through the bank sections. Verify the bank color, long names and aligned time columns.
5. Press Alt+H twice. Verify that empty banks disappear and return while their index entries remain.
6. Press Alt+Up and Alt+Down. Verify that the time division changes without changing the take.
7. Press F1, then Esc. Verify the mark legend and that the overview returns. Press F2 to return to Pads.

### Controller

1. Learn the overview action and the two view actions on MIDI buttons in F12.
2. Learn the empty-bank switch. Open the overview and change its view with those buttons. Use the existing bank controls to show another bank.
3. Hit and release a Gate pad. Verify that it sounds and stops as on Pads.

### Sound and views

Listen to the existing `renders/97-pattern-record.wav`. The overview adds no sound or new WAV. Inspect the overview catalogue files at 80x26 and 120x40.
