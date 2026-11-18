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

### Requirement: A verified PAD BANK press opens the current panel

A valid released-to-pressed status SHALL open the hidden panel on its main
page. A later valid press from the same Master port SHALL close its panel.
Each press SHALL act once. The press SHALL leave playback and modal controls
in place. It SHALL have no MIDI learn action.

#### Scenario: PAD BANK opens the panel once before a pad hit
- **WHEN** the shortcut is on and the controller gives a valid press
- **THEN** the panel opens before a pad hit
- **AND** the next command-layer hit acts once
- **AND** the next valid press from its Master closes the panel without a pad hit
- **AND** a new press opens the main page for the current screen

#### Scenario: PAD BANK respects input overlays
- **WHEN** the owner presses PAD BANK during a question, picker, Help, text entry or learning
- **THEN** the overlay and target keep priority
- **AND** the press has no MIDI learn action

### Requirement: Status delays preserve first-hit reveal and pad play

A status reply SHALL keep the current panel state if a playing or command pad
hit on that Master port is newer than the request. If the shortcut is off,
unavailable or late, the first command-layer hit SHALL reveal a hidden panel
without running a cell.

#### Scenario: Late PAD BANK status keeps both pad routes
- **WHEN** a command-layer hit arrives before its PAD BANK status reply
- **THEN** the hit can reveal the panel and the late reply leaves the panel in place
- **AND** a first-bank hit before the reply keeps the panel hidden and retains its release

#### Scenario: First-hit reveal survives a missing PAD BANK pulse
- **WHEN** the shortcut is off or a tap gives no usable reply
- **THEN** the first command-layer hit reveals the panel
- **AND** the next hit runs the visible cell once

### Requirement: Added context cells provide missing actions

Every panel view SHALL keep its main page, first context page and fixed bottom row. It SHALL add the common action cells in the table. Pads, Chop and Chop edit SHALL add the listed page 4 cells.

#### Scenario: Added cells reach their current actions
- **WHEN** the owner uses a listed cell in a screen or mode where its action works
- **THEN** the action has the same result as its keyboard path
- **AND** another screen or mode does not claim a route for a command that has no effect there

| View | Page | Cells in order: label (`action`) |
| --- | --- | --- |
| Every view | 2 | `output` (`record.output`), `play` (`transport.toggle`), `take` (`pad.take`), `open` (`project.pick`), `save` (`session.save`), `grid` (`ui.pads_toggle`), `learn` (`learn.toggle`), `undo` (`pattern.undo`), `knobs` (`pattern.controls`), `exit` (`app.quit`) |
| Every view | 3 | `erase` (`pattern.erase`), `record` (`pattern.record`) |
| Pads | 4 | `grid -` (`pattern.view_prev`), `grid +` (`pattern.view_next`), `rows` (`pattern.overview_empty_toggle`), `step` (`pattern.step_edit`) |
| Chop, Chop edit | 4 | `hits -` (`chop.hits_back`), `cut -` (`marker.delete`), `end -` (`slice.end_delete`), `order` (`chop.assign_order`) |

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
