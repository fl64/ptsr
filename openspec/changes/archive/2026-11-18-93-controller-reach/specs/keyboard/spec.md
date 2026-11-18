# keyboard Delta

## ADDED Requirements

### Requirement: Shared transport key keeps preview controls

The shipped keymap SHALL bind `Alt+U` to `transport.toggle` on every screen. On Files and Chop, Space SHALL keep its current preview action. `Alt+U` SHALL work in English and Russian layouts and SHALL toggle transport once per press. Text entry, questions, pickers and learning SHALL keep their input priority. An owner keymap entry for `Alt+U`, including `none`, SHALL keep priority over the shipped binding.

#### Scenario: Shared transport key keeps previews on Files and Chop
- **WHEN** separate keyboard runs press `Alt+U` and Space on Files and Chop in both supported layouts
- **THEN** `Alt+U` toggles transport once and Space keeps the screen's preview action
- **AND** the page 2 `trnspt` cell has the same transport result through raw SMC-PAD MIDI

#### Scenario: Owner transport key and modal input keep priority
- **WHEN** an owner map replaces `Alt+U` or sets it to `none`, or a text line, question, picker or learning mode owns input
- **THEN** the owner entry or active input mode keeps priority and transport does not toggle behind it

### Requirement: Exact Chop threshold keys work without kitty

On Chop and Chop edit, the shipped keyboard map SHALL bind `Ctrl+Alt+Up` to `chop.threshold_up` and `Ctrl+Alt+Down` to `chop.threshold_down`. The keys SHALL work with and without the kitty keyboard protocol. An owner binding SHALL keep priority.

#### Scenario: Exact Chop threshold keys resolve in both terminal modes
- **WHEN** the owner presses either threshold key with or without the kitty keyboard protocol
- **THEN** the same threshold action reaches Chop or Chop edit once per key press
- **AND** an owner binding for that key replaces the shipped action
