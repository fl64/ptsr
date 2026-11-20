## ADDED Requirements

### Requirement: Pad edit page cells

The pad edit panel SHALL have five page cells: `tone page`, `env page`, `filter page`, `source page` and `fx page` on device pads 9 to 13. A press SHALL open that page in one press. The cell of the current page SHALL be brighter than the other page cells. SMC-PAD LEFT and RIGHT without PAD BANK SHALL go around the pages as Tab and Shift+Tab do. LEFT on TONE SHALL open FX, and RIGHT on FX SHALL open TONE.

#### Scenario: The fx page cell opens FX in one press
- **WHEN** pad edit is open on TONE and the owner presses PAD BANK with device pad 13
- **THEN** the page is FX and the fx page cell is brighter than the other page cells

#### Scenario: LEFT on TONE opens FX
- **WHEN** pad edit is open on TONE, the panel is hidden and the owner presses LEFT
- **THEN** the page is FX

### Requirement: Pads keep a sound gain

Each pad SHALL have a sound gain in dB, 0 dB by default, from −30 to +30 dB. The engine SHALL apply it before the pad volume, with the same smoothing as the volume, so a change makes no click. The audio thread SHALL get the gain through the existing pad parameter path, with no allocation and no lock. The pad volume range SHALL stay −40 to +6 dB.

#### Scenario: The sound gain raises the level before the volume
- **WHEN** a pad has a sound gain of +12 dB and a volume of 0 dB and the offline render plays it
- **THEN** the render peak is 12 dB higher than with 0 dB gain, within 0.1 dB, with no clipping and no click

## MODIFIED Requirements

### Requirement: Undo on the pads

Ctrl+Z on Pads and Files SHALL undo one change per press through 100 steps:
- Volume, sound gain, pitch, envelope, filter, drive and sends.
- Mode, swap, copy, clear, reset, file assignment, source and shared FX.
A knob turn or value-key run on one parameter SHALL make one step for all touched pads.
Restored pads SHALL sound as before.

#### Scenario: Undo a knob turn
- **WHEN** the owner turns volume of Q from 0 to -12 dB in one turn and presses Ctrl+Z
- **THEN** Q is at 0 dB and sounds so after one press

#### Scenario: Undo over the scope
- **WHEN** all 16 pads are marked, the owner raises their cutoff and presses Ctrl+Z
- **THEN** all 16 cutoffs return and the status says `undo: cutoff on 16 pads`

#### Scenario: Undo a swap
- **WHEN** the owner swaps Q and V with Ctrl+X and presses Ctrl+Z
- **THEN** Q and V play their previous sounds with their settings

#### Scenario: Undo a clear
- **WHEN** the owner clears Q with Backspace and presses Ctrl+Z
- **THEN** Q has its file back and sounds as before

#### Scenario: Undo a shared slot
- **WHEN** the owner changes FX 2 from delay to reverb and presses Ctrl+Z
- **THEN** FX 2 holds the delay again with its time and feedback

### Requirement: Pad undo history and feedback

Undo status SHALL name what returned or say `nothing to undo`. It SHALL replace the notice of the undone action.
With nothing to undo, the SMC-PAD undo cell SHALL be grey.
Selection, banks, scope, pages and marks SHALL add no step.
History SHALL last until exit.

#### Scenario: Nothing to undo
- **WHEN** the program has just started and the owner presses Ctrl+Z on Pads
- **THEN** nothing changes and the status says `nothing to undo`

#### Scenario: What changes no pad
- **WHEN** the owner raises Q's volume, presses Ctrl+A twice, turns the bank, selects W
  and presses Ctrl+Z
- **THEN** Q's volume returns; bank, selection and marks stay as they are

#### Scenario: Undo links it back
- **WHEN** pad 7 plays slice 3, the owner moves its start on SOURCE and presses Ctrl+Z
- **THEN** pad 7 says `slice 3` and plays it again

#### Scenario: The undo notice replaces the action notice
- **WHEN** the owner clears pad 1, sees the clear notice and then presses undo
- **THEN** the status says `undo: clear on pad 1` and the clear notice is gone

### Requirement: Reset a pad to clean
In pad edit, `Backspace` SHALL reset scope pads to freshly loaded defaults: volume 0 dB, sound gain 0 dB, one-shot, pitch 0, default envelope, filter, drive, sends and mute group.
File and source (whole file, linked slice or own bounded piece) SHALL stay.
Empty pads SHALL be skipped.
Sounding pads SHALL reset without clicks.
Reset SHALL make one undo step.
Edit help SHALL name `Bksp reset`.

#### Scenario: A pad back to clean
- **WHEN** pad Q plays slice 3 with the pitch -5, the filter low-pass, the volume -6 dB and a sound gain of +5 dB,
  the owner opens the edit, presses `Backspace` and `Enter`
- **THEN** pad Q plays slice 3 with the pitch 0, the filter off, the volume 0 dB, the sound gain 0 dB, one-shot,
  and its top frame carries only `◻`
