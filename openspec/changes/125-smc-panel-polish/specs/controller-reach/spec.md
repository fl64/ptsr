## MODIFIED Requirements

### Requirement: Help has a controller route

More and P10 SHALL open Controller Help on each view. While Help shows, P7 and P8 SHALL
select its tabs. P9 and P10 SHALL scroll or move the selected control. Back
SHALL close Help. Help SHALL show the controls that work in its current tab.
A Help cell SHALL leave the screen behind Help in place. F1 and the keyboard
Help paths SHALL retain their results.

#### Scenario: Help cells navigate without acting behind Help
- **WHEN** the owner opens Help from the panel, uses P7 through P10 and then Back
- **THEN** only the Help tab, scroll position or selected controller control changes before Help closes
- **AND** the screen and its selected value remain unchanged

## ADDED Requirements

### Requirement: Verified USB SMC-PAD uses the saved PAD BANK preference

F12 MIDI SHALL offer `SMC-PAD PAD BANK` for a connected USB SMC-PAD after the
preset and encoder checks pass. It SHALL save `Off` or `Open panel`.
An absent setting SHALL use `Open panel`. A saved Off choice SHALL stay off.
Only `Open panel` on a verified unit SHALL start extra read-only status requests.

#### Scenario: PAD BANK starts open and preserves a saved Off choice
- **WHEN** the owner opens MIDI settings on a verified USB unit and sets Off
- **THEN** the setting and disabled shortcut survive restart
- **AND** a config without a saved choice uses Open panel
- **AND** an unverified unit or other device does not show the setting

## REMOVED Requirements

### Requirement: PAD BANK detection is opt-in for USB SMC-PAD

**Reason**: Verified USB uses Open panel by default.
**Migration**: See "Verified USB SMC-PAD uses the saved PAD BANK preference".
