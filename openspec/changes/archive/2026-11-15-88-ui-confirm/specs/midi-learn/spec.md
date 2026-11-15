# midi-learn Delta

## ADDED Requirements

### Requirement: Learn one shared confirm button

F12 MIDI SHALL list `ui.confirm` as a GLOBAL button action. The owner SHALL be able to learn a note or CC button for it. Learning SHALL keep the existing pending, save, cancel, clear, and restart rules. No factory profile SHALL gain a confirm binding in this change.

#### Scenario: Confirmation learning stays pending
- **WHEN** the owner selects `ui.confirm` in MIDI learning, moves a button, cancels or saves the pending binding
- **THEN** the learning steps do not answer an open question or change the underlying screen
- **AND** only a saved binding can confirm after learning closes

### Requirement: Learned confirmation uses the current input owner

While a question or picker is open, a learned confirm press SHALL reach it before a screen or edit binding. A release, repeated press, or disconnect SHALL NOT add another confirmation. Target checks and input order SHALL stay. Earlier pad releases SHALL reach their original pads.

#### Scenario: Learned confirm answers every current question
- **WHEN** a learned note or CC button sends raw MIDI for each current question type through the shipped profile and dispatch
- **THEN** each press answers only that question once through its existing target checks
- **AND** the button release does not answer again or play a pad

#### Scenario: Learned confirm selects in both pickers
- **WHEN** a learned confirm button sends raw MIDI while the path picker or project picker has a selected item
- **THEN** the picker uses only its current target and existing refusal checks
- **AND** no screen action below the picker runs

#### Scenario: Question confirmation wins over a screen binding
- **WHEN** the same controller button has GLOBAL `ui.confirm` and a screen or edit action, then a question opens
- **THEN** its press answers the question without running the screen or edit action
- **AND** outside the question the existing screen or edit action keeps its priority

#### Scenario: Confirm cannot consume an earlier pad release
- **WHEN** a Gate pad starts before a question, a learned confirm button answers it, and the original pad control releases
- **THEN** the original voice receives its release and no new pad or second confirmation starts
