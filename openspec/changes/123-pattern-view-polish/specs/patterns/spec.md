# patterns Delta

## ADDED Requirements

### Requirement: Overview step cells keep fixed width

Each overview step SHALL use three terminal cells at 80x26 and 120x40. A one-cell separator SHALL mark each beat. Straight and triplet views SHALL use whole beat groups. Step numbers SHALL align with their event cells. The system SHALL leave unused width blank rather than stretch cells. It SHALL retain beat shading, bar borders, the edit cursor, and the live playback plate.

#### Scenario: Overview fixed cells align every division
- **WHEN** the owner views each straight and triplet division at 80x26 and 120x40
- **THEN** each step number stays over a three-cell step and each beat starts after one separator
- **AND** bar, beat, cursor, and moving playback marks remain visible without changing A1

### Requirement: Overview shows three velocity levels

An occupied cell SHALL use low, medium, or high brightness for its highest visible velocity. The levels SHALL be 1–42, 43–84, and 85–127. A single on-grid hit SHALL retain `■`. A single off-grid hit SHALL retain `~`. Several hits SHALL retain their count mark. The step editor SHALL show the selected event's numeric velocity. Brightness SHALL NOT be the only mark that identifies a hit.

#### Scenario: Overview velocity levels keep event marks
- **WHEN** visible hits have velocities 1, 42, 43, 84, 85, and 127, including off-grid and multiple-hit cells
- **THEN** the three brightness levels follow those boundaries and the glyph still shows hit type or count
- **AND** the selected event shows its numeric velocity without changing raw rows

### Requirement: A1 step edit adds one hit at an empty cell

In step edit, a new keyboard or raw MIDI pad press on an empty cell SHALL add one raw A1 hit for that absolute pad. The hit SHALL use the first tick of the cursor cell and keep the received velocity. Shift plus a keyboard pad SHALL give full velocity. A OneShot pad SHALL store zero Gate ticks. A Gate pad SHALL store one current view-cell length. The edit press, repeat, and release SHALL make no immediate pad sound. Only the first press SHALL edit.

#### Scenario: Step input adds at an empty cursor without sound
- **WHEN** the owner presses a keyboard pad normally and with Shift, then a raw MIDI pad at an empty cursor cell
- **THEN** each accepted press adds one hit at that cell with its source velocity and no immediate sound
- **AND** repeats and releases add nothing, while ordinary pad play works after leaving edit

#### Scenario: Step input keeps Gate length and saved tick
- **WHEN** the owner adds OneShot and Gate pads at a fine view cell while capture quantize uses a coarse grid
- **THEN** both hits play at the selected cell, the OneShot has zero Gate ticks, and the Gate lasts one view cell
- **AND** changing view or quantize later does not move either saved hit

### Requirement: A1 step addition refuses unsafe work

The system SHALL check the full replacement before changing A1. It SHALL refuse at 4096 raw rows or when a conflicting input owner blocks editing. Refusal SHALL leave A1, Undo, dirty state, and sound unchanged. A new saved tick SHALL play at its stored position even when capture quantize differs from the view division.

#### Scenario: Step input refuses a full or stale edit
- **WHEN** A1 has 4096 rows, or a project or pattern change makes an edit target stale
- **THEN** the refused press adds no row and leaves history, dirty state, and sound unchanged
- **AND** the owner sees a next action without losing existing work

### Requirement: A1 step addition has one-event Undo and safe save

Each accepted addition SHALL create one common-history Undo step. Undo SHALL remove only that added row and SHALL not cross a newer unrelated edit. Save SHALL write the new raw row in format 5 without editor state. A save refusal SHALL keep the addition and Undo. Reopening SHALL restore the saved row with transport stopped and edit mode closed.

#### Scenario: Step addition Undo restores the prior A1
- **WHEN** the owner adds two hits, undoes the latest, and makes a newer pad or FX edit
- **THEN** each permitted Undo removes only its added row and does not cross the newer edit
- **AND** cursor movement, focus, repeats, and refused presses create no history

#### Scenario: Step addition saves without editor state
- **WHEN** an addition meets save refusal, then saves and the owner reopens the project
- **THEN** refusal keeps unsaved work and Undo, and the saved A1 contains the intended hit
- **AND** transport starts stopped, edit mode closes, and the view returns to 1/16

## MODIFIED Requirements

### Requirement: A1 overview shows every bank in one vertical list

The A1 overview SHALL keep all eight banks in its fixed index. By default, its vertical list SHALL show rows with visible hits and the selected pad in the current bank. It SHALL keep pad order within each bank. Each shown row SHALL show the pad key, pad name, and visible events at the selected view division. The view SHALL use compiled playback ticks without changing saved raw hits. It SHALL keep hidden raw hits hidden and show visible seam hits at the loop start.

#### Scenario: Overview shows occupied and selected pad rows
- **WHEN** A1 has hits on several pads and banks, including off-grid hits, two source hits in one cell, and a selected empty pad
- **THEN** the list shows only pads with visible hits and the selected pad in bank order
- **AND** each occupied cell shows its event type and count without merging source hits or changing A1

#### Scenario: Overview shows all pad rows and source events
- **WHEN** the owner uses Alt+H with hits on several pads and banks
- **THEN** each shown bank keeps all 16 pad rows in order, including empty rows
- **AND** occupied cells show their event type and count without changing A1

#### Scenario: Overview filters hidden hits before seam wrapping
- **WHEN** A1 has a hidden row beyond its shortened length and a visible row that quantizes to the seam
- **THEN** the hidden row does not appear in any overview cell or active-row count
- **AND** the visible seam row appears once at the loop start without changing the raw rows

### Requirement: Overview follows the shared view division and bank

The overview SHALL use the same runtime view division as the Pads strip. Existing bank controls SHALL move the viewport to the selected bank section and keep its selected row visible. Changing the division or current bank SHALL change only the overview projection and selection. It SHALL NOT change playback, quantize, raw hits, history, or dirty state. The view division SHALL return to 1/16 after a project change, as on Pads.

#### Scenario: Overview bank and view changes leave A1 unchanged
- **WHEN** the owner switches banks and view divisions while A1 plays, including a bank with no visible hits
- **THEN** the selected bank and pad appear in the viewport with the selected division, and A1 keeps its timing
- **AND** raw rows, quantize, history, and dirty state stay unchanged

### Requirement: Overview can hide empty bank sections

The overview SHALL show all eight bank numbers in a fixed index. The index SHALL show each bank with visible A1 hits. By default, the vertical list SHALL hide empty pad rows and bank sections, except the selected pad and current bank. The existing empty-view action, `Alt+H` by default, SHALL show all 16 rows in all eight bank sections. Another use SHALL restore the filtered view. A bank with only hidden raw hits SHALL count as empty in this view. This runtime toggle SHALL NOT change A1 or pad selection. The saved action ID SHALL stay `pattern.overview_empty_toggle`.

#### Scenario: Overview hides empty banks without losing access
- **WHEN** A1 has visible hits in two banks and hidden raw hits in another bank
- **THEN** the fixed index shows all eight banks while the list starts with occupied and selected rows only
- **AND** Alt+H shows all sections and rows, then restores the filtered view without changing A1

#### Scenario: Overview bank navigation reaches a hidden empty bank
- **WHEN** the owner selects a bank with no visible A1 hits while empty rows are hidden
- **THEN** that bank section appears with the selected pad row and the bank index marks it as current
- **AND** another bank selection restores the ordinary filtered view

### Requirement: A1 step correction removes one identified hit

An edit pad press SHALL target only its pad in the selected time cell. One visible event SHALL be removable on the first press. A cell with several events SHALL require an explicit event choice before removal. Repeats and releases SHALL remove nothing and make no immediate sound. An empty cell SHALL follow the addition rule.

#### Scenario: Step correction removes one hit without pad sound
- **WHEN** the owner presses a pad with one visible event at the selected cell
- **THEN** exactly that raw row disappears while other hits, pad settings, and held tails stay unchanged
- **AND** the press, repeats, and release make no new sound or removal

#### Scenario: Step correction chooses one of several hits
- **WHEN** the owner first presses another pad with several events in the cell, selects an event, and presses the pad again
- **THEN** the first press selects without removal and the next press removes only the chosen raw row
- **AND** other rows, including same-tick collisions, keep their timing, velocity, and Gate lengths

#### Scenario: Step correction rejects a stale event identity
- **WHEN** a project or pattern change invalidates a selected event before removal
- **THEN** the old choice cannot remove a replacement row with a reused index
- **AND** selection refreshes without an edit or sound in the new project

### Requirement: A1 step correction keeps ordered audio publication

The App SHALL check a replacement before mutation and publish each accepted addition or removal at an audio boundary. It SHALL keep transport and active note deadlines. Queue pressure SHALL retain accepted edits in order without callback allocation, blocking, or replay of past onsets.

#### Scenario: Step removal survives publication pressure
- **WHEN** removal and another ordered edit meet command or retirement pressure while A1 and Gate notes play
- **THEN** accepted edits remain whole and ordered, past hits do not replay, and started notes keep their tails
- **AND** the audio callback allocates, destroys, and blocks on no new work

#### Scenario: Step addition survives publication pressure
- **WHEN** addition and another ordered edit meet command or retirement pressure while A1 and Gate notes play
- **THEN** accepted edits remain whole and ordered, past hits do not replay, and started notes keep their tails
- **AND** the audio callback allocates, destroys, and blocks on no new work
