# projects Delta

## ADDED Requirements

### Requirement: Recorded patterns reuse format five

Completed pattern passes SHALL save through the existing format 5 raw-hit and settings fields. Runtime capture identity, count-in, erasure, pending gates and active recording SHALL NOT be stored. Load SHALL restore the raw take and settings with stopped transport and Idle recording. This change SHALL add no format migration or owner-project cleanup.

#### Scenario: Save and reopen a recorded rhythm without recording state
- **WHEN** a recorded and edited A1 saves and its project or scratch session reopens
- **THEN** raw rows, length, grid and strength match the completed take and transport remains stopped
- **AND** no recording phase or pending gate resumes and the saved format remains 5

### Requirement: Manual save completes the current pass

During pattern work, manual save SHALL first cancel CountIn or finish Recording and recover accepted capture through that boundary. It SHALL then invoke normal saving. It SHALL NOT report success before write success. Recording completion SHALL leave playback running. Count-in cancellation SHALL leave transport stopped. Refusal SHALL keep the completed take, Undo and pending save state.

#### Scenario: Manual save waits for a complete recording boundary
- **WHEN** Ctrl+S or a learned save button is pressed with recorded feedback and Gate completion pending
- **THEN** the pass finishes and saves its complete accepted rows before `Session saved` appears
- **AND** playback continues without resuming recording or writing runtime-only fields

A pending manual save SHALL keep its original input source and project ownership. Repeated save input SHALL join that request. A new pattern pass SHALL wait until the save attempt completes. Input processing, manual pads and releases SHALL continue while capture finishes. Autosave SHALL NOT replace the pending manual attempt. Project restoration SHALL discard an earlier project's request before it can write or report success.

#### Scenario: Repeated save waits for the same recording boundary
- **WHEN** the user requests another save while a manual save waits for delayed capture completion or a full command queue
- **THEN** only one save attempt and one pass Undo step result, with no early success notice
- **AND** manual pads and releases still work, a new recording pass waits, and the original request keeps its input source

#### Scenario: A delayed manual save cannot write another project
- **WHEN** project restoration replaces the source state before its pending manual save completes
- **THEN** the old request cannot write the new project or produce a save-success notice for it
- **AND** a later explicit save uses only the current project's state

The save request SHALL remove only its own progress notice when it completes or cancels. The progress notice SHALL NOT resume after the result notice expires. Removal of progress SHALL NOT change other notices or their existing display and history policies.

#### Scenario: Save completion removes hidden progress without hiding other notices
- **WHEN** another warning or error hides the progress notice before the pending manual save completes
- **THEN** completion removes only that request's progress notice, while other notices follow their existing display and history policies
- **AND** the success or refusal follows existing priority, while obsolete progress never resumes

#### Scenario: Save refusal retains a completed recording for retry
- **WHEN** a manual save ends a pass but read protection, a write failure or a missing folder refuses the write
- **THEN** the completed take, its Undo step and pending status remain and no false success appears
- **AND** an allowed retry saves the same recoverable work without recreating a missing project folder

### Requirement: Autosave and exit respect recording boundaries

Autosave SHALL wait during CountIn, Recording and Finishing. Quit SHALL complete or cancel that phase before applying existing exit-save policy. Audio loss SHALL permit recovery without waiting for a callback that cannot arrive. A write refusal SHALL retain existing warning and retry semantics. This change SHALL NOT promise recovery of unsaved data after a process crash.

#### Scenario: Autosave cannot write a partial gate take
- **WHEN** autosave time passes during count-in or recording with unfinished Gate notes
- **THEN** the last saved file stays unchanged and pending save status remains visible
- **AND** after the pass finishes ordinary autosave can save complete rows

### Requirement: Quit closes a recoverable pattern prefix

Quit SHALL freeze App capture at its last coherent accepted audio boundary. It SHALL complete recoverable rows without a new callback or acknowledgement. Later events SHALL NOT extend that prefix. Concurrent unavailable reads SHALL retry on normal UI turns without blocking input. Normal quit SHALL NOT set audio loss. A pending manual save SHALL keep its single attempt and source. Existing exit-save policy SHALL apply without an automatic retry after failure.

#### Scenario: Quit finalizes the recoverable pass before exit saving
- **WHEN** quit is requested during pattern work, with or without a final audio callback
- **THEN** recoverable capture is complete before existing exit-save policy runs
- **AND** no pending Gate duration or active recording phase enters the saved session

### Requirement: Project selection waits until pattern work ends

Project picker entry, direct open and creation SHALL refuse active phases, a requested start or pending quit before target validation, source saving or filesystem changes. Refusal SHALL preserve the take and clock and explain how to finish first. After completion, existing Idle transition rules SHALL apply. New-project state SHALL NOT accept capture, releases or controls from an earlier project or pass.

#### Scenario: A project request cannot interrupt an active take
- **WHEN** keyboard or raw MIDI requests project selection during any active recording phase
- **THEN** the current take, screen and clock remain and the notice explains how to finish recording
- **AND** after completion the picker uses existing transition safety without carrying recording state into another project
