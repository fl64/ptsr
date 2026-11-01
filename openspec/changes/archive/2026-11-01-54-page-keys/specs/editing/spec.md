## MODIFIED Requirements

### Requirement: Parameter pages
The system SHALL group the parameters of the edited object into pages of at most 3; knob k
turns the k-th parameter of the current page, on every screen. In the edit, `Tab` turns the
page forward and `Shift+Tab` back, in a ring, and the pick lands on the first parameter of
the new page; `I` `K` (`↑` `↓`) walk all parameters of the object one by one and the page
follows the picked parameter. A knob bound to the page (`edit.page`) splits its travel
into equal parts, one per page in this order, and turns to the page under it at once, no
pickup; the pick lands on the first parameter of that page, and a turn that stays inside
the part of the current page keeps the pick. An object with one page keeps it on `Tab`
and on the page knob. The pads screen
has TONE (volume, pitch), ENV (attack, decay, release), FILTER (filter, cutoff, reso) and
SOURCE (source, start, end), in this order; the chop screen has BOUNDS (slice start, slice
end).

#### Scenario: The envelope page fills three knobs
- **WHEN** the picked parameter of a pad is attack, decay or release
- **THEN** the page is ENV and the knobs turn attack, decay and release, in this order

#### Scenario: Tab turns the page
- **WHEN** the pad edit is open on the pitch of TONE and the owner presses `Tab` twice
- **THEN** the page is FILTER, the filter type is picked, and knob 1 turns it

#### Scenario: Shift+Tab turns back
- **WHEN** the pad edit is open on TONE and the owner presses `Shift+Tab`
- **THEN** the page is SOURCE and the source is picked

#### Scenario: A knob picks the page
- **WHEN** the pad edit is open on TONE and the page knob is turned to the middle of its
  travel, then a little further inside the same part
- **THEN** the page is FILTER at once, the filter type is picked, and the second turn keeps
  it

#### Scenario: One page
- **WHEN** the slice edit is open on the slice end and the owner presses `Tab`
- **THEN** the page stays BOUNDS and the slice end stays picked

#### Scenario: Up down walks the parameters
- **WHEN** the edit mode is open and the owner presses `K` and then `I`
- **THEN** the next parameter is picked and then the previous one, and the page follows
  each pick

#### Scenario: The filter page
- **WHEN** the owner presses `K` past the end of the ENV page
- **THEN** the page is FILTER and the knobs turn the filter type, the cutoff and the
  resonance, in this order
