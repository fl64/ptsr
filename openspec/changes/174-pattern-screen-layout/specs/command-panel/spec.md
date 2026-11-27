## MODIFIED Requirements

### Requirement: The screens view opens screens and global actions

Row 1 of the screens view SHALL show pads, files, chop and pattern. The pattern cell SHALL open the Pattern screen. Row 2 SHALL show fx, mixer, setup and help. Row 3 SHALL show save, open and exit.
Mixer SHALL stay dim until it exists. Help SHALL open Controller Help.
Global actions SHALL appear only in the screens view.
The screens view SHALL NOT show learn, because control bindings change from the keyboard only.

#### Scenario: Global actions live in the screens view
- **WHEN** the owner visits the screens view and every action page of each current view
- **THEN** save, open and exit appear only in the screens view
- **AND** no action page shows play, help or learn

#### Scenario: Setup keeps Settings open on a second selection

- **WHEN** the owner selects P11 Setup, changes the Settings tab and selects Setup again
- **THEN** both selections show the Settings actions and keep Settings open
- **AND** the second selection keeps the current tab
- **AND** F12 still closes Settings
