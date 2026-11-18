## ADDED Requirements

### Requirement: Holding PAD BANK shows the current panel

With the hold shortcut on a verified USB SMC-PAD, the panel SHALL show while the status reads pressed and hide on the first released status.
Each hold SHALL show the actions of the current screen or mode.
A hold SHALL leave playback and modal controls in place. It SHALL have no MIDI learn action.

#### Scenario: The panel shows only while PAD BANK is held
- **WHEN** the owner holds PAD BANK, hits one cell and releases PAD BANK during playback
- **THEN** the panel shows during the hold and the cell acts once
- **AND** the panel hides on release and playback continues

#### Scenario: PAD BANK respects input overlays
- **WHEN** the owner holds PAD BANK during a picker, Help, text entry or learning
- **THEN** the overlay keeps priority and the panel stays hidden
- **AND** the hold has no MIDI learn action

### Requirement: The PAD BANK hold routes pad hits

With the hold shortcut on, a pad hit SHALL act by the hold, not by the note bank of the device.
While held, it SHALL run the cell at the same physical position. Otherwise, it SHALL play the pad.
A release SHALL go to the target of its press.

#### Scenario: A pad hit follows the hold, not the device layer
- **WHEN** the device sends second-bank notes without a hold and first-bank notes during a hold
- **THEN** the hits without a hold play their pads at the same physical positions
- **AND** the hits during the hold run the cells at those positions

#### Scenario: A pad release keeps the target of its press
- **WHEN** the owner releases PAD BANK while a cell pad is down, then releases that pad
- **THEN** the panel hides and the pad release runs no cell and plays no sound

### Requirement: Status delays keep pad play

With the hold shortcut on, a hit before the first pressed reply of a hold SHALL play the pad.
A failed status read or a preset change SHALL act as a release.
If the shortcut is off or unavailable, the first second-bank hit SHALL reveal a hidden panel without running a cell.

#### Scenario: A hit before the hold reply plays the pad
- **WHEN** a pad hit arrives after the PAD BANK press but before its pressed reply
- **THEN** the pad plays once and the next hit during the hold runs a cell

#### Scenario: A failed status read hides the panel
- **WHEN** a status read fails or the preset changes during a hold
- **THEN** the panel hides and the next pad hit plays

#### Scenario: First-hit reveal works without the hold shortcut
- **WHEN** the shortcut is off or the unit is not verified
- **THEN** the first second-bank hit reveals the panel
- **AND** the next hit runs the visible cell once

### Requirement: Verified USB SMC-PAD saves the PAD BANK hold preference

F12 MIDI SHALL offer `SMC-PAD PAD BANK` for a connected USB SMC-PAD after the
preset and encoder checks pass. It SHALL save `Off` or `Hold panel`.
An absent setting SHALL use `Hold panel`. A saved Off choice SHALL stay off.
Only `Hold panel` on a verified unit SHALL start extra read-only status requests.

#### Scenario: PAD BANK starts on hold and preserves a saved Off choice
- **WHEN** the owner opens MIDI settings on a verified USB unit and sets Off
- **THEN** the setting and disabled shortcut survive restart
- **AND** a config without a saved choice uses Hold panel
- **AND** an unverified unit or other device does not show the setting

## REMOVED Requirements

### Requirement: A verified PAD BANK press opens the current panel

**Reason**: Change 131 shows the panel only while PAD BANK is held.
**Migration**: `Holding PAD BANK shows the current panel` and `The PAD BANK hold routes pad hits`.

### Requirement: Status delays preserve first-hit reveal and pad play

**Reason**: A toggle no longer exists, so a late reply cannot leave the panel in the wrong state.
**Migration**: `Status delays keep pad play`.

### Requirement: Verified USB SMC-PAD uses the saved PAD BANK preference

**Reason**: The setting value changes from `Open panel` to `Hold panel`.
**Migration**: `Verified USB SMC-PAD saves the PAD BANK hold preference`.
