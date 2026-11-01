## ADDED Requirements

### Requirement: Every binding acts
The system SHALL bind keys and controls only to actions it has. A binding in the keymap,
the owner's keymap or a device profile that names an action the program does not have is
named on the status line at start with its file, key and name; the binding does nothing
and the rest loads. MIDI bindings in the owner's config that name no action are dropped
at start, the config is saved without them, and the status says how many went.

#### Scenario: The shipped bindings all act
- **WHEN** the program loads its keymap and the X-Session Pro profile
- **THEN** every binding names an action the program has; `Ctrl+K` is free and knobs
  k6–k10 of the chop screen are free

#### Scenario: An unknown action is named
- **WHEN** the owner's keymap binds `Ctrl+G` to `pad.flyaway`
- **THEN** the status names the file, `Ctrl+G` and `pad.flyaway`, and every other key works

#### Scenario: Dead bindings leave the config
- **WHEN** the owner's config binds X-Session `cc 29` on the chop screen to `slice.pitch`
- **THEN** at start the binding is dropped, the config no longer has it, the status says
  one binding to a missing action was dropped, and the next start says nothing
