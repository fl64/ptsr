## ADDED Requirements

### Requirement: Panic gives an instruction and result

The effective keyboard Panic key SHALL appear in Help. The SMC-PAD help SHALL name its two STOP presses. After the first press the screen SHALL show how to complete or cancel the gesture. After Panic the screen SHALL report the stop. Safety messages SHALL keep their space.

#### Scenario: Panic instructions fit both terminal sizes

- **WHEN** the owner opens the panel and arms Panic at 80x26 or 120x40
- **THEN** the next STOP and the cancel route are visible with the safety messages

## MODIFIED Requirements

### Requirement: Help explains current SMC-PAD panel roles

With the SMC-PAD connected, F1 SHALL show the current panel view and 16 physical cell labels. It SHALL show PLAY and STOP roles and the current PAD BANK rule: a hold, or the first second-bank hit without the shortcut. It SHALL name unavailable cells as unavailable. It SHALL describe Panic on two STOP presses with the panel open. It SHALL not describe SHIFT as a ptsr modifier. The keyboard help and X-Session view SHALL keep their paths.

#### Scenario: Help names panel entry and current cells

- **WHEN** the owner opens F1 before and after the panel shows the screens view
- **THEN** F1 explains how PAD BANK shows the panel and lists the visible cells in physical order
- **AND** it names Redo as unavailable, gives the Panic gesture, and keeps keyboard commands visible
