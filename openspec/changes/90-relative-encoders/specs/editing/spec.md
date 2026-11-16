# editing Delta

## ADDED Requirements

### Requirement: Relative parameter steps match value keys

A relative knob bound to a parameter SHALL use the unaccelerated detent in the change design. That detent SHALL match a keyboard value-key tap for pad, slice, effect and pattern values. It SHALL keep the active page, selection, scope and limits. A relative knob SHALL not show a pickup direction.

#### Scenario: One detent matches one value key
- **WHEN** separate keyboard and raw MIDI runs start with the same selected pad and parameter
- **AND** the owner taps L in one run and turns a relative knob one slow detent in the other
- **THEN** the same parameter value and pad scope result from both runs
- **AND** the MIDI run has no pickup direction

#### Scenario: Relative edit slot follows its page
- **WHEN** `edit.param_1` is bound to a relative knob and the owner changes from ENV to FILTER
- **THEN** its next detent changes the filter kind, not the earlier attack value
- **AND** a slot absent from the current page does nothing

### Requirement: One relative message makes one Undo step

A turn with multiple decoded or accelerated steps SHALL make one Undo step. Each pad in the scope SHALL keep its own limit. A step at a limit SHALL not report a changed value.

#### Scenario: Fast value turn is one Undo step
- **WHEN** one rapid relative CC message moves a scoped value by more than one stored step
- **THEN** each pad in the scope reaches its own limit without crossing it
- **AND** one Undo returns the entire turn

### Requirement: Relative page and tempo steps use their current modes

`edit.page` SHALL move one page per unaccelerated detent as Tab or Shift+Tab does. Pads pages SHALL wrap. Chop's single BOUNDS page SHALL stay selected. FX SHALL select its existing two slots. Tempo SHALL move by 1 BPM per detent, or 0.1 BPM while the owner holds fine mode. A relative step SHALL not bypass an open question or pattern-control ownership.

#### Scenario: Relative page matches Tab
- **WHEN** the owner starts on TONE and sends one positive and one negative relative page step
- **THEN** the selected page follows the same order and first-parameter rule as Tab and Shift+Tab
- **AND** a later pad hit keeps its normal route

#### Scenario: Fine tempo changes a tenth
- **WHEN** tempo starts at 120 BPM and a relative tempo knob sends one positive step with and without fine mode
- **THEN** the two separate runs reach 120.1 BPM and 121 BPM respectively
- **AND** tempo stays within 40 to 300 BPM
