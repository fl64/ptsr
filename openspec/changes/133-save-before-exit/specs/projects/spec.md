## MODIFIED Requirements

### Requirement: Quit closes a recoverable pattern prefix

Quit SHALL freeze App capture at its last coherent accepted audio boundary.
Quit SHALL complete recoverable rows without a new callback or acknowledgement.
Later events SHALL NOT extend that prefix.
Concurrent unavailable reads SHALL retry on normal UI turns without blocking input.
Normal quit SHALL NOT set audio loss.
A pending manual save SHALL keep its single attempt and source.
The exit rules below SHALL apply after capture completion.

#### Scenario: Quit finalizes the recoverable pass before exit saving

- **WHEN** the owner requests Exit during pattern work, with or without a final audio callback
- **THEN** recoverable capture is complete before exit saving
- **AND** no pending Gate duration or active recording phase enters the saved session

#### Scenario: A failed exit keeps the completed A1 take

- **WHEN** Exit finalizes a recoverable A1 take and the required save fails
- **THEN** the app keeps the completed take, its Undo step and pending work
- **AND** a later retry saves the accepted hits and Gate lengths without resuming recording

## ADDED Requirements

### Requirement: Exit saves pending work before closing

An exit request SHALL wait for A1 capture to finish or cancel.
Exit SHALL wait for a pending manual save of the current session.
Exit SHALL close a clean session.
If pending work remains after a save refusal, Exit SHALL show the recovery question.
Otherwise Exit SHALL save pending work once and close only after success.

#### Scenario: Exit closes a clean session without writing

- **WHEN** the owner requests Exit with a clean session and no pending manual save
- **THEN** ptsr closes without writing the session file

#### Scenario: A successful exit saves the pending edit

- **WHEN** the owner requests Exit with pending edits and the required save succeeds
- **THEN** ptsr closes after the write
- **AND** reopening the session restores the edits

#### Scenario: A prior save refusal stops exit

- **WHEN** pending work remains after a save refusal and the owner requests Exit
- **THEN** ptsr stays open and shows the recovery question with the last reason
- **AND** Exit waits for an explicit retry, even if storage is available again

#### Scenario: Exit uses the result of a pending manual save

- **WHEN** the owner requests Exit while a manual save of the current session waits for A1
- **THEN** Exit waits for that save and keeps its original input source
- **AND** success closes the saved session, or refusal opens recovery with the completed take

### Requirement: Exit recovery explains a save refusal

If Exit cannot save pending work, ptsr SHALL cancel the exit request and keep the session open.
The recovery question SHALL show the session name, the refusal reason, `Retry Save` and `Back`.
It SHALL show the effective controls at 80x26 and 120x40.
Existing question confirm and cancel actions SHALL select `Retry Save` and `Back` respectively.

#### Scenario: A failed exit save keeps the session open

- **WHEN** the first required save for Exit fails
- **THEN** ptsr stays open with the current project, edits and Undo history
- **AND** the saved file keeps its previous bytes

#### Scenario: Exit recovery shows the reason and controls

- **WHEN** the exit recovery question is visible at 80x26 or 120x40
- **THEN** it names the project or Scratch, explains the refusal and shows both actions
- **AND** it names the applicable keyboard or SMC-PAD controls

#### Scenario: Back leaves exit recovery with the edits

- **WHEN** the owner chooses Back in the recovery question
- **THEN** the question closes and the owner returns to the same screen
- **AND** the edits and save status remain

### Requirement: The exit questions use physical SMC-PAD numbers

Both questions SHALL show the pad numbers printed on the device.
P11 SHALL cancel and P12 SHALL confirm from either pad bank without a PAD BANK hold.

#### Scenario: The printed SMC-PAD numbers answer both exit questions

- **WHEN** Exit or its recovery question is visible and the owner presses P11 or P12 in either SMC-PAD bank
- **THEN** the question shows the printed pad numbers
- **AND** P11 cancels or P12 confirms once, even with the PAD BANK shortcut off or unavailable

### Requirement: Retry Save uses one manual save

`Retry Save` SHALL use the existing manual save path for the current session.
Repeated requests during a pending retry SHALL join that attempt.
Success SHALL show `Session saved` and keep ptsr open.
Refusal SHALL keep the work and show the current reason in the recovery question.
The owner SHALL request Exit again to close ptsr.

#### Scenario: A successful retry returns to the saved session

- **WHEN** the owner restores write access and chooses Retry Save
- **THEN** the retry saves the pending edits and shows `Session saved`
- **AND** ptsr stays open until another Exit request

#### Scenario: A refused retry keeps the recovery question

- **WHEN** Retry Save cannot write because of a missing folder, read protection or a write failure
- **THEN** ptsr stays open with pending work and shows the current reason
- **AND** an existing file retains its bytes and a missing folder stays absent

#### Scenario: Repeated retry input keeps one save

- **WHEN** the owner repeats Retry Save before its save attempt completes
- **THEN** the app keeps one save attempt for the same session
- **AND** the result appears once and ptsr stays open
