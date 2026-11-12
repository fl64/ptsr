# Spec Delta

## ADDED Requirements

### Requirement: A1 step correction needs an explicit mode

The owner SHALL enter correction only from the A1 overview with an existing A1. Entry and exit SHALL leave transport, raw hits, dirty state and history unchanged. A conflicting input owner or pending capture SHALL refuse entry without ending its work.

#### Scenario: Step correction entry keeps A1 playing
- **WHEN** the owner requests correction with absent, empty, stopped and playing A1
- **THEN** absent A1 gives the effective record action, while existing A1 enters without moving transport
- **AND** first entry selects the valid playback cell or the first cell, and reentry retains the same-project cursor

#### Scenario: Step correction refuses conflicting work
- **WHEN** capture, recovery, placement, marking, pad edit, pattern controls, audio recording or a question owns input
- **THEN** correction refuses with a next action and does not change the existing work
- **AND** text, pickers, learning and original pad releases keep their priority

### Requirement: A1 step correction removes one identified hit

An edit pad press SHALL target only its pad in the selected time cell. An empty cell SHALL select without insertion. One visible event SHALL be removable on the first press. A cell with several events SHALL require an explicit event choice before removal. Repeats and releases SHALL remove nothing.

#### Scenario: Step correction removes one hit without pad sound
- **WHEN** the owner presses a pad with one visible event at the selected cell
- **THEN** exactly that raw row disappears while other hits, pad settings and held tails stay unchanged
- **AND** the press, repeats and release make no new sound or removal

#### Scenario: Step correction chooses one of several hits
- **WHEN** the owner first presses another pad with several events in the cell, selects an event and presses the pad again
- **THEN** the first press selects without removal and the next press removes only the chosen raw row
- **AND** other rows, including same-tick collisions, keep their timing, velocity and Gate lengths

#### Scenario: Step correction rejects a stale event identity
- **WHEN** a project or pattern change invalidates a selected event before removal
- **THEN** the old choice cannot remove a replacement row with a reused index
- **AND** selection refreshes without an edit or sound in the new project

### Requirement: A1 step cursor uses the shared view division

The edit cursor SHALL keep its own time cell and page while playback continues. Bank changes SHALL keep time and select the new bank. A view change SHALL select the new cell that contains the old cell's first tick. Cursor and view changes SHALL NOT edit A1.

#### Scenario: Step cursor stays independent of playback
- **WHEN** A1 crosses beats, pages and the loop seam while the owner moves the cursor or changes banks and view division
- **THEN** the cursor stays on its chosen page and the shared view remains aligned to complete beats
- **AND** A1 timing, raw rows, history and dirty state stay unchanged

### Requirement: A1 step correction keeps ordered audio publication

The App SHALL check a replacement before mutation and publish it at an audio boundary. It SHALL keep transport and active note deadlines. Queue pressure SHALL retain accepted edits in order without callback allocation, blocking or replay of past onsets.

#### Scenario: Step removal survives publication pressure
- **WHEN** removal and another ordered edit meet command or retirement pressure while A1 and Gate notes play
- **THEN** accepted edits remain whole and ordered, past hits do not replay and started notes keep their tails
- **AND** the audio callback allocates, destroys and blocks on no new work

### Requirement: A1 step removal has one-event Undo and safe save

Each successful removal SHALL create one common-history Undo step. Pattern Undo SHALL respect newer unrelated edits. Save SHALL write corrected raw rows in format 5 without editor state. A save refusal SHALL keep the correction and Undo.

#### Scenario: Step removal Undo restores one hit
- **WHEN** the owner removes two hits, undoes the latest and makes a newer pad or FX edit
- **THEN** each permitted Undo restores only its identified hit and does not cross the newer edit
- **AND** cursor movement and empty presses create no history or redo

#### Scenario: Step correction saves without editor state
- **WHEN** a correction meets save refusal, then saves and the owner reopens the project
- **THEN** refusal keeps unsaved work and Undo, and the saved raw rows contain only the intended correction
- **AND** transport starts stopped, edit mode closes and view division returns to 1/16
