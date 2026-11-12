# keyboard Delta

## MODIFIED Requirements

### Requirement: Recording key roles

Alt+R SHALL record the microphone and Ctrl+T the output. Ctrl+R SHALL toggle pattern recording through `pattern.record` on every current screen. The commands SHALL remain distinct and explicit owner bindings SHALL retain priority.

#### Scenario: Ctrl C copies instead of quitting
- **WHEN** Pads receives Ctrl+C in raw mode, with or without the extended keyboard protocol
- **THEN** `pad.copy` is resolved, no quit is requested and Ctrl+Q remains the quit command

#### Scenario: Ctrl R records the pattern on every screen
- **WHEN** default Ctrl+R arrives on Pads, Files, Chop, FX or Settings outside a blocking input mode
- **THEN** it starts count-in when stopped, starts overdub when playing or ends the current recording
- **AND** no microphone recording starts and no screen changes

### Requirement: Reserved keyboard chords

The shipped map SHALL leave Ctrl+Y and Ctrl+. free until redo and panic in 93a. Ctrl+R SHALL run pattern recording in 97. Chop Ctrl+Y MAY explain its former role. Ctrl+B SHALL keep the Files favorite action. Bake SHALL NOT take it in this change. Owner bindings SHALL remain allowed to use reserved chords.

#### Scenario: Future commands stay unbound
- **WHEN** the default map is loaded after pattern recording ships but before redo and panic
- **THEN** Ctrl+Y and Ctrl+. execute no action, except a moved-key notice where defined
- **AND** Ctrl+R records patterns and Ctrl+B still changes the favorite on Files

### Requirement: Moved key notices

The App SHALL keep Chop Ctrl+N to Ctrl+G, Chop Ctrl+Y to H, and Pads J/L to U/O move notices. Only an old press without an applicable shipped binding or explicit owner entry SHALL show its Info notice. It SHALL NOT run the former action. Bound J/L SHALL change a value without a move notice. Ctrl+R SHALL no longer produce the microphone move notice.

#### Scenario: Old microphone key names its replacement
- **WHEN** Ctrl+R is unbound through an explicit owner removal after pattern recording ships
- **THEN** no recording or microphone move notice appears

#### Scenario: Old normalize key names its replacement
- **WHEN** Ctrl+N is unbound on Chop and the owner presses it
- **THEN** the flash says `normalize moved to Ctrl+G` and no pad gain changes
- **AND** Ctrl+N on Pads still opens the pad name input

#### Scenario: Old hit chop key names its replacement
- **WHEN** Ctrl+Y is unbound on Chop and the owner presses it
- **THEN** the flash says `chop by hits moved to H` and no cuts change

#### Scenario: Rebound bank keys change values without a move notice
- **WHEN** Pads uses the default J/L bindings and the owner presses J or L
- **THEN** the selected parameter changes and no moved-bank notice appears

## ADDED Requirements

### Requirement: Full help page controls preserve ordinary input

Alt+PageUp/PageDown SHALL page the F1 key table through GLOBAL `help.page_prev` and `help.page_next` commands.
The commands SHALL support learned buttons without factory assignments.
Presses and keyboard repeats SHALL navigate, but releases SHALL NOT.
Owner bindings, `none`, input ownership, ordinary controls and Messages SHALL retain priority.
Outside the key table, the commands SHALL NOT change the screen, bank, selection or recording state.

#### Scenario: Help page keys preserve ordinary roles
- **WHEN** default or owner page keys arrive on each current screen with either keyboard protocol and edit state
- **THEN** only the open F1 key table changes page on presses or repeats, and releases do not navigate
- **AND** owner bindings and `none` in the global or active edit layer retain priority, while ordinary bank keys, pads and Messages controls retain their behavior

#### Scenario: Help page commands keep first-frame order
- **WHEN** keyboard or learned page commands follow help opening before its first drawn frame
- **THEN** the first frame applies those commands in order with the current wrapped bounds
- **AND** a tab change preserves the key-table offset, while closing and reopening discards old page requests

#### Scenario: Learned help pages retain input ownership
- **WHEN** raw MIDI uses learned page buttons with F1 open, closed or blocked by text, questions, pickers or learning
- **THEN** only an unblocked F1 key table changes page, without changing recording or the underlying screen
- **AND** button releases and earlier pad releases retain their original ownership

### Requirement: Zoom while holding a bound modifier

Chop SHALL support repeating Ctrl/Alt+I/K zoom in either layout, with slice edit open or closed.
Releases SHALL NOT zoom or move bounds.
Tempo chords, other screens, blocking modes, explicit owner bindings and `none` SHALL retain their behavior.
Legacy Tab SHALL retain its action. Alt+I SHALL replace Ctrl+I when the terminal cannot distinguish Ctrl+I from Tab.

#### Scenario: Modified zoom keys preserve routing and owner overrides
- **WHEN** Chop receives Ctrl/Alt+I/K in either layout, including repeats and releases
- **THEN** it resolves zoom presses and repeats without changing tempo bindings or other screens
- **AND** owner overrides and `none` remain effective, while legacy Tab retains its previous action

### Requirement: Pattern control keys

Alt+N SHALL toggle compact pattern controls. I/K and Up/Down SHALL select Bars, Quantize or Strength. J/L and Left/Right SHALL change the selected value. Enter, Esc or Alt+N SHALL close controls. Otherwise, unclaimed Esc SHALL finish Recording or cancel CountIn. Alt+E SHALL toggle recording erasure. Alt+Z SHALL request pattern-pass Undo globally. Ctrl+Z on Pads SHALL undo through common history. Plain and Shift pad keys SHALL remain pads.

#### Scenario: Adjust a pattern without opening a step grid
- **WHEN** Alt+N opens controls on any current screen and direction keys change each parameter
- **THEN** those keys edit the shown pattern parameter without changing underlying screen parameters
- **AND** closing controls restores the original screen, bank, selection and edit page

#### Scenario: Pattern keys work in both layouts and protocols
- **WHEN** recording, controls, erasure and pattern Undo keys arrive in English or Russian with extended or legacy key input
- **THEN** their logical actions agree and plain or Shift pad keys never execute those commands
- **AND** repeats or releases do not toggle recording, controls or erasure again

### Requirement: Pattern defaults preserve owner keys

Explicit owner entries and `none` SHALL override new defaults without rewriting files. Hidden recording defaults SHALL use existing startup shadow warnings. Owner Ctrl+R mapped to `record.mic` SHALL keep microphone hold or toggle behavior. Effective help SHALL name the actual recording keys or `none`.

#### Scenario: Owner recording overrides retain their old meaning
- **WHEN** an owner binds Ctrl+R to microphone or `none` and overrides another new pattern key
- **THEN** those entries remain effective and startup names each hidden default once
- **AND** the owner file stays unchanged and Ctrl+R shows no obsolete move notice

### Requirement: Pattern commands respect active input ownership

Text, pickers, questions and learning SHALL retain their existing source-specific priorities. Selecting a pattern command for learning SHALL NOT execute it. Pending placement, marking, Chop assignment and project restoration SHALL refuse recording start. Recording SHALL refuse entry into pad-consuming placement and marking modes. Earlier releases SHALL keep their original pad addresses.

#### Scenario: Pattern commands cannot escape a blocking input mode
- **WHEN** pattern commands arrive during text, pickers, questions, learning or project restoration
- **THEN** existing input ownership remains and no underlying recording command executes
- **AND** permitted original releases still finish their sounds

#### Scenario: Placement and recording cannot consume the same pad
- **WHEN** recording starts with placement, marking or Chop assignment active, or those modes are requested during recording
- **THEN** the incompatible request refuses with a next-step instruction
- **AND** no pad is replaced, marked or accidentally recorded by that request

#### Scenario: A recording pad release keeps its original bank and screen
- **WHEN** a held pad crosses a bank change, screen change, record end or erasure toggle before release
- **THEN** the release reaches its original sound ownership and cannot create a new onset
- **AND** neither the new bank nor a newer recording pass receives the old captured release
