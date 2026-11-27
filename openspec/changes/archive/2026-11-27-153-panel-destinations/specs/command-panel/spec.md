## MODIFIED Requirements

### Requirement: The screens view opens screens and global actions

Row 1 of the screens view SHALL show pads, files, chop and A1. Row 2 SHALL show fx, mixer, setup and help. Row 3 SHALL show save, open and exit.
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

### Requirement: Each view has at most 12 actions

A screen or mode SHALL have one action page with at most 12 actions in cells 1 through 12.
Cells 1–4 SHALL be device pads 13–16, cells 5–8 device pads 9–12 and cells 9–12 device pads 5–8.
An action that does not fit SHALL go to its own screen or mode.
The loader SHALL reject a second action page with a notice.
The shipped actions SHALL match the table. A dash is an empty cell.

#### Scenario: Shipped action pages match the table
- **WHEN** the layout test reads each view of the shipped panel file
- **THEN** each view has one page with the labels and actions of the table by device pad

#### Scenario: A second action page is rejected
- **WHEN** a panel file gives a view a second action page
- **THEN** a notice names the file and view, and the first page keeps its cells

| View | P5–P8 | P9–P12 | P13–P16 |
| --- | --- | --- | --- |
| Pads | edit pad, copy pad, chop pad, clear pad | swap pads, mark pad, mark bank, play mode | pitch gate, record output, hide pads, strip view |
| Pad edit | play mode, copy pad, pitch gate, reset pad | tone page, env page, filter page, source page | fx page, —, —, pin sample |
| A1 | record A1, edit steps, erase hits, tap tempo | steps wider, steps finer, A1 setup, clear A1 | hide empty, —, —, — |
| Files | load to pad, take file, listen file, chop file | star folder, —, —, — | —, —, —, — |
| Chop | cut here, cut by hits, fewer hits, equal parts | put on pads, fill pads, edit slice, trim silence | remove all cuts, loop play, play range, cursor step |
| Chop edit | norm, join left, join right, hear as pad | —, —, —, — | —, —, —, — |
| Settings | choose, —, —, — | —, —, —, — | —, —, —, — |

### Requirement: State cells and their states

The panel SHALL show a state for these cells only.

| Cell | States |
| --- | --- |
| play range | to end, to cut, slice |
| loop play | on, off |
| cursor step | even, hits, cuts |
| hear as pad | on, off |
| play mode | 1 shot, gate |
| pitch gate | on, off |
| mark pad | on, off |
| strip view | Sound, Rhythm |
| pin sample | on, off |
| hide empty | shown, hidden |
| listen file | on, off |
| star folder | on, off |

#### Scenario: Every state cell shows one of its states

- **WHEN** the owner holds PAD BANK on each view with a state cell
- **THEN** each cell of the table shows one of its listed states, and no other cell shows a state
