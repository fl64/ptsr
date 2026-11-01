## ADDED Requirements

### Requirement: Undo on the pads
The system SHALL undo the last change of the pads with `Ctrl+Z` on the pads and files
screens, one step a press, back through the last 100 steps: a parameter of a pad (volume,
pitch, envelope, filter, drive, sends), the mode, a swap, a clear, a file put on a pad,
the source of a pad, a shared effect slot. A knob turn or a run of value keys on one
parameter is one step, on every pad it touched. An undone pad sounds as it did before the
step. The status line says what came back (`undo: volume on 16 pads`), or `nothing to
undo`. The selection, the bank, the scope, the pages and the marks change no pad and are
no steps. The history lives until the program closes.

#### Scenario: Undo a knob turn
- **WHEN** the owner turns the volume knob of pad Q from 0 to −12 dB in one turn and
  presses `Ctrl+Z`
- **THEN** pad Q is at 0 dB again and sounds so, after one press

#### Scenario: Undo over the scope
- **WHEN** the scope is the bank, the owner raises the cutoff of the 16 pads and presses
  `Ctrl+Z`
- **THEN** every one of the 16 pads has its cutoff back, and the status says
  `undo: cutoff on 16 pads`

#### Scenario: Undo a swap
- **WHEN** the owner swaps pads Q and V with `Ctrl+X` and presses `Ctrl+Z`
- **THEN** Q and V play what they played before, with their settings

#### Scenario: Undo a clear
- **WHEN** the owner clears pad Q with `Backspace` and presses `Ctrl+Z`
- **THEN** pad Q has its file back and sounds as before

#### Scenario: Undo a shared slot
- **WHEN** the owner turns FX 2 from a delay to a reverb and presses `Ctrl+Z`
- **THEN** FX 2 holds the delay again with its time and feedback

#### Scenario: Nothing to undo
- **WHEN** the program has just started and the owner presses `Ctrl+Z` on the pads screen
- **THEN** nothing changes and the status says `nothing to undo`

#### Scenario: What changes no pad
- **WHEN** the owner raises the volume of pad Q, then presses `Ctrl+A` three times, turns
  the bank and selects pad W, and presses `Ctrl+Z`
- **THEN** the volume of pad Q comes back; the bank, the selection and the scope stay as
  they are

#### Scenario: Undo links it back
- **WHEN** pad 7 plays slice 3, the owner moves its start on the SOURCE page and presses
  `Ctrl+Z`
- **THEN** pad 7 says `slice 3` again and plays slice 3
