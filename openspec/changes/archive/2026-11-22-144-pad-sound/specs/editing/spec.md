## ADDED Requirements

### Requirement: Pad edits keep the sound shape

Copy, swap, Undo and Redo SHALL retain the pad sound settings and the existing scope of each edit.

#### Scenario: Copy and undo preserve the pad sound shape

- **WHEN** the owner copies a shaped pad and then changes and undoes its settings
- **THEN** the copy has the same sound shape and the earlier settings return on Undo

## MODIFIED Requirements

### Requirement: Parameter pages
The system SHALL group edited parameters into pages of at most four.
On every screen, knob k SHALL turn parameter k of the current page.
Pads SHALL use these pages in order:
- TONE: volume, pitch, mute group.
- ENV: attack, decay, release, hold.
- FILTER: filter, cutoff, reso.
- SOURCE: source, start, end.
- FX: drive, send 1, send 2.
- SOUND: pan, fine tune, velocity, curve.
Chop SHALL use BOUNDS: slice start, slice end.

#### Scenario: The envelope page fills three knobs
- **WHEN** the picked parameter of a pad is attack, decay or release
- **THEN** the page is ENV and the knobs turn attack, decay, release and hold in this order

### Requirement: Parameter page keys
In edit, and on Pads with edit closed, `Tab` SHALL select the next page and `Shift+Tab` the previous page, cyclically.
A page change SHALL select its first parameter.
An object with one page SHALL keep its page and selected parameter on `Tab`.

#### Scenario: Tab turns the page
- **WHEN** the pad edit is open on the pitch of TONE and the owner presses `Tab` twice
- **THEN** the page is FILTER, the program selects the filter type, and knob 1 turns it

#### Scenario: Shift+Tab turns back
- **WHEN** the pad edit is open on TONE and the owner presses `Shift+Tab`
- **THEN** the page is SOUND and the program selects Pan

### Requirement: Parameter page knob
The `edit.page` knob SHALL divide its travel equally among pages in their defined order.
It SHALL select the page at its position immediately, without pickup, and select that page's first parameter.
A turn within the current page's part SHALL keep the selected parameter.
An object with one page SHALL keep that page on the page knob.

#### Scenario: A knob picks the page
- **WHEN** the pad edit is on TONE and the owner turns the page knob just below its midpoint
- **AND** the owner turns it further inside the same part
- **THEN** the page is FILTER (the third of six) at once, the program selects the filter type, and the
  second turn keeps it

#### Scenario: One page
- **WHEN** the slice edit is open on the slice end and the owner presses `Tab`
- **THEN** the page stays BOUNDS and the slice end stays picked

### Requirement: The screen shows the layer
The system SHALL always show the active layer, selected object and selected parameter.
Edit SHALL change the edited panel's frame colour and name the mode and all pages in its title, with the current page lit.
The pad title SHALL use `EDIT · TONE ENV FILTER SOURCE FX SOUND`.
A title wider than the panel SHALL omit `EDIT ·` first.
If the page names still do not fit, show the current page and its position.
Frame colour SHALL still identify edit mode.

#### Scenario: The frame tells the mode
- **WHEN** the owner presses `Enter` on a pad and then `Esc`
- **THEN** the pad panel frame changes colour and its title says
  `EDIT · TONE ENV FILTER SOURCE FX SOUND` with TONE lit, and after `Esc` the frame is
  as before

#### Scenario: The title shows where Tab leads
- **WHEN** the pad edit is open on TONE and the owner presses `Tab` until SOUND
- **THEN** the lit page in the title moves through every page in order, the other pages
  stay named in it
