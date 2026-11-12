# Spec Delta

## ADDED Requirements

### Requirement: A1 correction routes new pad presses through the UI

Keyboard and raw MIDI pad presses accepted in correction SHALL reach the App silently with their original pad, bank, source and project. A sounding Gate from before correction SHALL keep its original release across mode, bank and screen changes. Delayed edit presses SHALL never become sound after exit.

#### Scenario: Silent edit presses keep original sound ownership
- **WHEN** a keyboard or raw MIDI Gate crosses correction entry, exit, bank and screen changes
- **THEN** the earlier sounding Gate releases its original voice and an edit press stays silent
- **AND** its repeat, release and delayed delivery cannot remove another hit or play a new-bank pad

### Requirement: A1 correction follows ordered controller commands

A learned edit or cursor command SHALL take effect before a later raw MIDI pad press in the same production stream. Invalid focus and stale project input SHALL not edit the current A1. Ordinary direct pad performance SHALL resume after correction closes.

#### Scenario: Raw MIDI correction keeps command order
- **WHEN** raw MIDI opens correction, selects a cell and presses its pad before redraw
- **THEN** the pad addresses only that selected cell without immediate sound or recording
- **AND** its release cannot repeat removal or become a sounding pad

#### Scenario: Raw MIDI correction rejects stale work
- **WHEN** a project transition, invalid focus or delayed mode change precedes a controller pad event
- **THEN** stale edit input cannot remove current-project rows or sound after exit
- **AND** accepted direct Gate releases keep their original sound owner
