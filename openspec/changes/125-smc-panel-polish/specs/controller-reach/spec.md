## MODIFIED Requirements

### Requirement: Help has a controller route

More and P10 SHALL open Help on each view. While Help shows, P7 and P8 SHALL
select its tabs. P9 and P10 SHALL scroll or move the selected control. Back
SHALL close Help. Help SHALL show the controls that work in its current tab.
A Help cell SHALL leave the screen behind Help in place. F1 and the keyboard
Help paths SHALL retain their results.

#### Scenario: Help cells navigate without acting behind Help
- **WHEN** the owner opens Help from the panel, uses P7 through P10 and then Back
- **THEN** only the Help tab, scroll position or selected controller control changes before Help closes
- **AND** the screen and its selected value remain unchanged
