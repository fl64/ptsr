## RENAMED Requirements

- FROM: `### Requirement: Banks on J and L`
- TO: `### Requirement: Keyboard bank controls`

## MODIFIED Requirements

### Requirement: Pad edit mode

The system SHALL open edit for the selected pad with Enter on Pads; Enter or Esc SHALL
close it. Pages and parameters SHALL follow editing, "Parameter pages". Tab and
Shift+Tab SHALL turn pages forward and back. I/K and Up/Down SHALL pick the previous
or next parameter with the page following the pick. Tab, I/K and J/L with their arrow
forms SHALL work with edit open or closed; J/L and Left/Right SHALL change the selected
parameter. U/O SHALL turn banks in both states, as "Keyboard bank controls" says.

#### Scenario: The page follows the picked parameter
- **WHEN** the owner enters pad edit and presses K until decay is picked
- **THEN** the page is ENV and the screen shows what the knobs turn

#### Scenario: The knobs turn the page
- **WHEN** pad edit is open on ENV and the owner turns knob 2
- **THEN** decay of the selected pad or change scope follows the knob with soft takeover

#### Scenario: Value keys work with pad edit closed
- **WHEN** Pads is open with edit closed, volume is selected and the owner presses L
- **THEN** volume rises by 1 dB for the change scope, selection and bank stay, and edit stays closed

#### Scenario: Value arrows work with pad edit closed
- **WHEN** Pads is open with edit closed and the owner presses Left or Right
- **THEN** the selected value changes as with J or L, without turning banks

### Requirement: Keyboard bank controls

The system SHALL turn banks with PageUp/PageDown and Ctrl+1…8 on every screen with
pads, including in edit. On Pads, U/O SHALL also turn banks with edit open or closed,
keeping the selected pad position. J/L and Left/Right SHALL change values, not banks,
in the shipped map. Owner keymaps that explicitly bind those keys to bank actions
SHALL keep their bindings. Controller bank actions SHALL stay unchanged. Neither
bank keys nor arrows SHALL choose another pad position; a pad hit chooses it.

#### Scenario: L turns to the next bank
- **WHEN** the owner explicitly maps L on Pads to `bank.next` and presses it on bank 1
- **THEN** bank 2 opens with the same pad position selected

#### Scenario: The next bank while editing a pad
- **WHEN** edit of pad W in bank 1 is open and the owner presses PageDown
- **THEN** bank 2 opens, edit stays open on W and J/L still change its selected value

#### Scenario: The same keys on the files screen
- **WHEN** bank 1 is open on Files and the owner presses PageDown
- **THEN** bank 2 is shown on the grid and the file list does not move

#### Scenario: U and O turn banks on Pads
- **WHEN** Pads is on bank 1 with edit closed and the owner presses O, then U
- **THEN** bank 2 opens and then bank 1, with the selected position unchanged

#### Scenario: U and O turn banks during pad edit
- **WHEN** pad edit is open and the owner presses O or U
- **THEN** the bank turns, edit stays open and J/L keep changing the selected value

### Requirement: Undo on the pads

The system SHALL undo the last pad change with Ctrl+Z on Pads and Files, one step per
press, back through 100 steps: pad parameters (volume, pitch, envelope, filter, drive,
sends), mode, swap, copy, clear, reset, file assignment, source and shared effect slots.
One copy placement SHALL restore the target's full previous state in one undo step
without changing its source. A cancelled copy SHALL add no step. A knob turn or run of
value keys on one parameter SHALL be one step on every pad touched. An undone pad
SHALL sound as before. The status SHALL name what came back or say `nothing to undo`.
Selection, banks, scope, pages and marks SHALL add no step. History SHALL last until exit.

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

#### Scenario: Undo one pad copy
- **WHEN** the owner copies Q onto occupied W after confirmation and presses Ctrl+Z
- **THEN** W's previous sound, source link, name and all settings return in one step
- **AND** Q stays unchanged throughout
