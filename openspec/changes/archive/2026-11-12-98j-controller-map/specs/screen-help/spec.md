## ADDED Requirements

### Requirement: F1 controller tab

F1 SHALL have a `Controller` tab after `Messages`. It SHALL draw the `[layout]` of one connected device inside the F1 frame.
Each control SHALL show a label of not more than 6 cells. The label SHALL be the action that the control does now on the screen under F1, with the open edit layer and the learned bindings.
A free control SHALL show `-`. A binding to an action that the program does not have does nothing, so it SHALL show `-` too. Each control kind SHALL have its own brackets.

#### Scenario: Controller tab draws the device top
- **WHEN** the X-Session Pro is connected and the owner opens the F1 `Controller` tab at 80x26 or 120x40
- **THEN** the tab draws each control of its layout at its place with its kind
- **AND** each label is the short label of the action that the same control does through the MIDI dispatch

#### Scenario: Controller labels follow the screen and the edit
- **WHEN** the owner opens the `Controller` tab on F2, on F3, and on F2 with the pad edit open
- **THEN** each label shows the action of that screen and layer
- **AND** a learned binding shows over the profile action, and a free control shows `-`

#### Scenario: Every action has a short label
- **WHEN** the program lists all its actions
- **THEN** each action has an ASCII short label of 1 to 6 cells

### Requirement: Controller tab selection

On the `Controller` tab, the arrows SHALL select the nearest control in their direction. Empty places SHALL be skipped.
A line under the drawing SHALL give the control name, its kind, its action and the action's full label.
The selection SHALL NOT send an action or change a value. Pad presses and releases under F1 SHALL keep their route.

#### Scenario: Arrows select controls on the drawing
- **WHEN** the owner moves the selection with the arrows across the X-Session layout
- **THEN** the selection skips empty places and stops at the edges
- **AND** the line under the drawing names the selected control and its action

#### Scenario: Selection does not act
- **WHEN** the owner moves the selection while a controller pad is held
- **THEN** no action, value change or sound command comes from the selection
- **AND** the pad release reaches its pad

### Requirement: Controller device pages

The `Controller` tab SHALL show one device. A top line SHALL name each connected device and mark the current one.
The ports of one ALSA client with one profile SHALL be one device. Two devices of one profile SHALL show as `<name> 1` and `<name> 2` in the order of connection.
On `Controller`, the help page actions SHALL change the device. The tab SHALL start on the device of the last MIDI event, or on the first connected device.

#### Scenario: Help page keys change the device
- **WHEN** two devices are connected and the owner presses Alt+PgDn on the `Controller` tab
- **THEN** the tab shows the other device and the top line marks it
- **AND** Alt+PgDn on the `Help` tab still pages the key table

#### Scenario: Tab starts on the last used device
- **WHEN** the owner turns a knob on the second device and then opens the `Controller` tab
- **THEN** the tab shows the second device

#### Scenario: Two devices of one profile have numbers
- **WHEN** two devices with the same profile are connected
- **THEN** the top line names them with the numbers 1 and 2 in the order of connection

#### Scenario: Ports of one device share a page
- **WHEN** the X-Session Pro connects with its ports `MIDI 1` and `MIDI 2`
- **THEN** the top line names one X-Session Pro without a number

#### Scenario: Device disconnects while its page is open
- **WHEN** the shown device disconnects while the `Controller` tab is open
- **THEN** the tab shows the next connected device, or the empty state when no device is left
- **AND** a device that connects later gets a place on the top line without a change of the current page

### Requirement: Controller tab without a drawing

A device without a valid layout that fits the frame SHALL show a list of its controls and their actions. Up and Down SHALL move the selection, and the list SHALL scroll with it.
Without a connected device, the tab SHALL show `No MIDI device is connected.` and the names of the loaded profiles.

#### Scenario: Profile without a layout shows a list
- **WHEN** a device without `[layout]` is connected and the owner opens the `Controller` tab
- **THEN** the tab lists each control with its action, and the selection scrolls the list

#### Scenario: Layout too wide for the frame shows a list
- **WHEN** a layout row has more places than the frame width allows
- **THEN** the tab shows the list for that device

#### Scenario: No device shows the known profiles
- **WHEN** no MIDI device is connected and the owner opens the `Controller` tab
- **THEN** the tab shows `No MIDI device is connected.` and the names of the loaded profiles

## MODIFIED Requirements

### Requirement: Local F1 page controls

Within F1, Tab SHALL select the next page and Shift+Tab the previous page of Help, Messages and Controller.
On Messages, Up/Down and PageUp/PageDown SHALL scroll entries. On Controller, the arrows SHALL select controls. F1/Esc SHALL close help.
These local controls SHALL NOT change global bindings or values behind the overlay.
Existing pad press/release behavior under help SHALL remain.
This change SHALL NOT invent a factory controller binding for history navigation.

#### Scenario: F1 navigation stays inside the overlay
- **WHEN** the owner changes F1 pages or scrolls Messages at either supported size
- **THEN** the visible page and history position change without moving parameters or banks behind help
- **AND** F1/Esc closes help while ordinary controls outside help retain their roles

### Requirement: Stable F1 window

Help, Messages and Controller SHALL use the same frame across the available body between status, notice and bottom help rows.
At a fixed terminal size, frame position and size SHALL NOT depend on the page, help rows or message count.
All page labels and the shared page/close hints SHALL retain their positions. Only Messages SHALL advertise its scroll controls.
Terminal resizing SHALL adapt all pages to the same body without changing existing message scroll rules or input behavior.

#### Scenario: F1 pages keep stable geometry
- **WHEN** the owner switches between Help and empty or populated Messages at 80×26 or 120×40
- **THEN** both pages keep the same frame corners and dimensions, including after help content or history changes
- **AND** status, notice and bottom help rows remain outside that frame

#### Scenario: F1 controls keep their positions
- **WHEN** the owner switches F1 pages or new messages arrive
- **THEN** both page labels and the Tab/Shift+Tab and F1/Esc hints stay at the same positions
- **AND** the selected page remains distinct while only Messages shows its existing scroll hints

#### Scenario: F1 frame follows terminal size
- **WHEN** the owner enlarges and reduces the terminal with either F1 page open
- **THEN** both pages use the same current body and keep their frame within the terminal
- **AND** Messages retains its existing wrapping, page height and bounded scrolling

#### Scenario: Controller page keeps the F1 frame
- **WHEN** the owner switches to and from the Controller page at 80×26 or 120×40
- **THEN** the frame corners, the page labels and the Tab/Shift+Tab hint stay at the same positions as on Help and Messages
- **AND** the F1/Esc hint stays at its position on Help
