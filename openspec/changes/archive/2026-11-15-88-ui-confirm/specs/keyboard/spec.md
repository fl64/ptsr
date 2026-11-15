# keyboard Delta

## ADDED Requirements

### Requirement: One confirm action for questions and pickers

The shipped Enter key SHALL use `ui.confirm` while a question or picker is open. That owner SHALL receive it before the screen or edit layer. Each press SHALL confirm at most once. Releases SHALL NOT confirm. Esc and existing question answers SHALL keep their roles. Confirmation SHALL keep target and save checks.

#### Scenario: Enter confirms every current question
- **WHEN** the owner opens each current question type and presses Enter through production keyboard input
- **THEN** only the named question answers once through its existing target checks
- **AND** no action of the screen or edit layer below the question runs

#### Scenario: Enter confirms both current pickers
- **WHEN** the owner selects a folder in the path picker or an existing project in the project picker and presses Enter
- **THEN** the picker uses the selected target and keeps its existing success or refusal result
- **AND** the underlying screen does not run its Enter action

#### Scenario: Enter uses a typed project name
- **WHEN** the owner types a new name in the project picker and presses Enter
- **THEN** the picker uses that exact name with its existing save and creation checks
- **AND** a refusal keeps the typed name available

### Requirement: Confirm keeps owner bindings and input priority

An explicit owner keymap entry, including `none`, SHALL override the shipped Enter binding. Text input, learning, and original pad releases SHALL keep priority over confirmation.

#### Scenario: A removed Enter binding stays removed
- **WHEN** the owner sets the applicable Enter binding to `none` and opens a question
- **THEN** Enter does not confirm or run an underlying action
- **AND** an existing effective question answer or learned `ui.confirm` button can still answer
