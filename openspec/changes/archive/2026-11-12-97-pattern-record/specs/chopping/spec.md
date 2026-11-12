# chopping Delta

## MODIFIED Requirements

### Requirement: Listen mode

In Chop listen mode, bank actions SHALL change banks. Until a slice of this file lies on a pad, the grid SHALL act as a slice keyboard outside pattern work. Key k SHALL play and select slice 16·b + k, where b is the selected bank. The row SHALL name the slice range and cells SHALL draw slice waves. Pattern controls or an active recording phase SHALL instead show and play real pads of the chosen bank.

#### Scenario: Before the layout
- **GIVEN** a sample has 8 slices, none lies on a pad and no pattern work is active
- **WHEN** the owner hits the third pad key on Chop
- **THEN** slice 3 plays and is selected, the grid names the cells `slice 1`…`slice 8`, and the row over it says `slices 1–8`

#### Scenario: Record real pads while an unassigned chop is open
- **WHEN** an unassigned chop is open and pattern count-in, recording or pattern controls becomes active
- **THEN** the grid names real pads and pad input plays those assignments, not an unassigned slice preview
- **AND** only accepted recording-phase hits enter A1 and ending pattern work restores the slice keyboard

### Requirement: Banks on the chop screen

Chop SHALL change banks with PgUp, PgDn and Ctrl+1…8 in listen, assignment and slice edit. Until a slice lies on a pad, those keys SHALL page the slice keyboard outside pattern work. During pattern controls or active recording, those keys SHALL select real pad banks. The open chop and earlier recorded addresses SHALL remain unchanged.

#### Scenario: The next bank while chopping
- **WHEN** bank 1 is shown on the chop screen in the listen mode and the owner presses PgDn
- **THEN** bank 2 is shown and the cursor stays where it was

#### Scenario: The same keys while assigning
- **WHEN** the assign mode is open on bank 1 and the owner presses PgDn
- **THEN** bank 2 is shown, as in the listen mode

#### Scenario: Pages of the keyboard of slices
- **WHEN** a sample is cut into 20 slices, none lies on a pad, no pattern work is active and the owner presses PgDn
- **THEN** the row over the grid says `slices 17–20` and the first pad key plays slice 17

#### Scenario: Chop recording uses absolute pad banks
- **WHEN** recording on Chop changes from bank 1 to bank 8 and both banks receive manual pad hits
- **THEN** A1 stores the original absolute addresses and both banks keep their real pad assignments
- **AND** a later release retains its original bank address

### Requirement: The chop screen opens in silence

Outside an active pattern recording phase, opening Chop SHALL fade sounding pad voices without a click. Preview SHALL remain untouched and listen-mode pad hits SHALL keep their slice behavior. Entering Chop during CountIn, Recording or Finishing SHALL NOT hush the rhythm or change recording phase. Leaving Chop SHALL stop no pad voices.

#### Scenario: A long pad fades on entry
- **WHEN** a one-shot pad plays a long sample and the owner presses F3 outside an active pattern recording phase
- **THEN** the pad fades out without a click as the chop screen opens

#### Scenario: Slices still play after the hush
- **WHEN** the chop screen opened with the pads hushed and the owner hits a pad outside pattern work
- **THEN** the slice of that pad plays, as in the listen mode

#### Scenario: Enter Chop during recording without interrupting the rhythm
- **WHEN** F3 or its learned action opens Chop during pattern count-in, recording or finishing
- **THEN** existing pad voices and the clock continue without a new hush or recording restart
- **AND** real pad hits retain their accepted capture or warm-up roles

## ADDED Requirements

### Requirement: Zoom without releasing the bound modifier

On Chop, the owner SHALL be able to move a slice bound and zoom without releasing Ctrl or Alt.
Ctrl/Alt+I SHALL zoom in and Ctrl/Alt+K SHALL zoom out around the cursor.
Zoom SHALL preserve slice boundaries, the cursor and the selected slice.
It SHALL NOT change which bound Ctrl or Alt addresses.
Existing assignment and higher-priority input modes SHALL continue to block underlying zoom actions.

#### Scenario: Zoom between bound edits with the modifier held
- **WHEN** the owner holds Ctrl or Alt, moves a bound, presses and holds I, presses K and moves the bound again
- **THEN** the view zooms in and out without a modifier release, with slice edit open or closed
- **AND** zoom leaves the cursor, selection and boundaries unchanged, while each bound command edits only its original bound

### Requirement: Chop previews remain separate from pattern recording

Space, cursor audition and slice preview SHALL retain their preview roles during pattern work. Their audio SHALL NOT become recorded pad events. Slice selection from real linked-pad hits SHALL retain its existing behavior. After pattern work ends, new pad input SHALL return to normal Chop routing while earlier holds keep their original releases.

#### Scenario: Preview a slice without recording a pattern hit
- **WHEN** pattern recording runs on Chop and Space or cursor audition plays a slice
- **THEN** that preview does not add a raw row or change the recording clock
- **AND** an actual linked-pad hit still records its absolute pad and selects its matching slice
