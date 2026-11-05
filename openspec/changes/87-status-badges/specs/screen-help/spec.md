## MODIFIED Requirements

### Requirement: Manual save feedback

Each manual save request SHALL show `Session saved` for five seconds after success, or an Error notice with the current refusal reason after failure.
Ctrl+S and a learned `session.save` button SHALL use the same action and feedback.
Successful autosave SHALL stay silent.
Notice lifetime or acknowledgement SHALL NOT clear pending work, read protection or persistent save failure.

#### Scenario: The manual save speaks
- **WHEN** the owner presses Ctrl+S and the save succeeds
- **THEN** the notice line shows `Session saved` for five seconds before returning to the hint unless another notice takes priority

#### Scenario: The autosave is silent
- **WHEN** the session autosaves successfully after an idle period
- **THEN** no message shows and the hints and screen stay as they were

#### Scenario: Repeated manual refusals each explain the reason
- **WHEN** the owner requests manual saves twice while the write problem or read protection remains
- **THEN** each request shows its current refusal as an Error and adds its own history entry
- **AND** pending changes remain and no `Session saved` result appears

#### Scenario: Keyboard and controller save feedback agree
- **WHEN** keyboard and raw MIDI request the same save through Ctrl+S and a learned `session.save` button
- **THEN** both report the same success, write refusal or read-protection result and notice level
- **AND** both retain the same pending work and saved bytes after failure

### Requirement: Copy help stays distinct from swap

A pending copy SHALL show `COPY · hit a pad` and cancellation instructions, distinct from swap, without moving the three-row help band.
The shared mode badge SHALL identify copy even when compact help omits its detail.
Existing copy placement, confirmation and cancellation actions SHALL remain unchanged.

#### Scenario: Copy in hand is visible without a mode redesign
- **WHEN** a copy is taken in hand on Pads in a small or large terminal
- **THEN** `COPY · hit a pad` or its compact COPY badge and cancel instructions are visible without implying swap
- **AND** grid, panel and strip rows stay fixed when the help changes

## ADDED Requirements

### Requirement: Notice levels and origins

Transient notices SHALL use Done, Info, Warn or Error and record Keyboard, the actual Device, or System as origin.
They SHALL use `✓` cyan, `·` muted, `!` magenta and `✗` bold magenta respectively.
Existing transient and startup messages SHALL enter the same model without inventing device identity.
Text-input prompts, ongoing recording/loading state and persistent save status SHALL retain their separate precedence and meaning.

#### Scenario: Notice appearance identifies its level
- **WHEN** a notice of each level appears in a separate fixture
- **THEN** its glyph, text and style identify the level without relying on color alone
- **AND** the stored origin identifies the keyboard, actual device or system

#### Scenario: Input prompts retain typing priority
- **WHEN** a filter, name or picker text line is open while background notices arrive
- **THEN** its prompt and cursor stay usable and pad keys still type without playing
- **AND** the notices enter history without replacing the text input

#### Scenario: Startup warnings enter history once
- **WHEN** startup produces warnings or errors and frames redraw repeatedly
- **THEN** each produced notice enters history once with System origin
- **AND** rendering recording time or loading progress does not flood history

### Requirement: Notice lifetimes and owner acknowledgement

Done and Info SHALL last three seconds of visible time, Warn six seconds and manual save success five seconds.
Error SHALL wait for the next new owner press or control movement, without consuming that action.
Releases, MIDI clock, background events and redraws SHALL NOT acknowledge Error.
The no-key-releases startup warning SHALL retain its existing first-keypress acknowledgement rule.

#### Scenario: Timed notices use their visible lifetime
- **WHEN** controlled time reaches a Done, Info, Warn or manual save-success deadline
- **THEN** the notice ends at three, three, six or five seconds respectively and the next notice or normal hint appears

#### Scenario: Hidden notices retain their visible time
- **WHEN** an editor prompt or a higher-priority state hides a timed notice
- **THEN** its visible lifetime pauses until the notice line can show it again
- **AND** new notices still enter history while that line is unavailable

#### Scenario: A delayed update preserves the next notice lifetime
- **WHEN** an update finds an expired notice with another notice pending
- **THEN** the next notice starts its remaining visible lifetime at that update
- **AND** time before that update does not expire a notice that never appeared

#### Scenario: Owner acknowledgement keeps the original action
- **WHEN** an Error is visible and the owner presses a keyboard key, pad or control
- **THEN** that action acknowledges the Error and still follows its normal production route exactly once
- **AND** persistent save state and existing pad-release behavior remain unchanged

#### Scenario: Background and release events cannot dismiss errors
- **WHEN** an Error is visible and note-off, CC release, MIDI clock, loader completion or timer events arrive
- **THEN** the Error stays until a new owner press or control movement
- **AND** required releases and background work still complete normally

### Requirement: Notice priority and bounded queue

Notice priority SHALL be Error, Warn, Done, Info.
A lower level SHALL wait behind a live higher level. An equal level SHALL replace it. A higher level SHALL interrupt it.
Interrupted notices SHALL retain remaining visible time.
Pending storage SHALL hold at most 50 notices and discard the oldest at its lowest level on overflow, never the active Error.
Queue removal SHALL NOT remove an entry from the separate history.

#### Scenario: An informational result cannot hide an error
- **WHEN** an Error is visible and an Info or Done notice arrives without a new owner action
- **THEN** the Error stays visible and the lower-level notice waits
- **AND** both results remain in message history

#### Scenario: Interrupted notices retain remaining time
- **WHEN** a higher-level notice interrupts a timed notice and later ends
- **THEN** the interrupted notice resumes with only its remaining visible time
- **AND** an equal-level replacement does not replay the replaced notice

#### Scenario: Queued notices retain priority and production order
- **WHEN** a notice ends with several notices pending
- **THEN** the highest pending level appears first, with older notices first within that level
- **AND** interruption does not change a notice's original production order

#### Scenario: Expired notices do not resume after interruption
- **WHEN** a higher-level notice arrives exactly at or after the current timed notice's deadline
- **THEN** the expired notice does not enter the pending queue or appear again
- **AND** its history entry remains eligible for the latest 50 entries

#### Scenario: Pending notice overflow remains bounded
- **WHEN** more than 50 pending notices accumulate behind an active Error
- **THEN** the active Error remains and pending storage stays within 50
- **AND** overflow removes the oldest lowest-level pending notice while history retains its own latest 50 entries

### Requirement: F1 message history

F1 SHALL open normal help first and provide a Messages page with the latest 50 notices, newest first, including level and origin.
History SHALL be in memory only and empty after restart.
Messages SHALL remain readable and scrollable at both supported catalogue sizes.
Leaving help SHALL restore its prior screen without editing work, dirtying the session or issuing sound commands.

#### Scenario: F1 retains recent results in order
- **WHEN** more than 50 notices arrive and the owner opens F1 Messages
- **THEN** exactly the latest 50 appear newest first with their levels and origins
- **AND** expired and replaced notices remain eligible for that history

#### Scenario: F1 history does not persist or edit work
- **WHEN** the owner reads and closes Messages, then restarts
- **THEN** reading did not change pads, session dirtiness, project or audio commands
- **AND** the new process starts with no previous-process history

### Requirement: Local F1 page controls

Within F1, Tab/Shift+Tab SHALL select Help or Messages.
On Messages, Up/Down and PageUp/PageDown SHALL scroll entries. F1/Esc SHALL close help.
These local controls SHALL NOT change global bindings or values behind the overlay.
Existing pad press/release behavior under help SHALL remain.
This change SHALL NOT invent a factory controller binding for history navigation.

#### Scenario: F1 navigation stays inside the overlay
- **WHEN** the owner changes F1 pages or scrolls Messages at either supported size
- **THEN** the visible page and history position change without moving parameters or banks behind help
- **AND** F1/Esc closes help while ordinary controls outside help retain their roles

### Requirement: Truthful question controls

An existing question SHALL show ASK, its current target and effective keyboard answer controls.
A controller answer SHALL appear only for an actual connected binding that reaches that answer through current production dispatch.
Unavailable controller paths SHALL be omitted, not inferred from a template or future `ui.confirm` action.
Question safety, existing answers and non-answer input behavior SHALL stay unchanged.

#### Scenario: Questions show effective working answers
- **WHEN** an existing question opens with default keys, owner key overrides or a working connected controller binding
- **THEN** its answer line names only controls that actually confirm or cancel that question
- **AND** recorded keyboard/raw-MIDI runs through advertised paths retain the existing result and refusal behavior

#### Scenario: Missing controller confirmation stays honest
- **WHEN** a controller has no existing binding that answers the current question
- **THEN** the question shows its keyboard answers without claiming STOP, Apply or another unavailable controller path
- **AND** no new action or binding is created by drawing the answer line
