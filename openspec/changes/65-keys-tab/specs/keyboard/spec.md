## ADDED Requirements

### Requirement: Keys on F12
The system SHALL show on the settings screen (F12) a Keys tab with every action a key can
do, by group as the MIDI tab has them, each with its keys, the owner's own marked. `Enter`
on an action waits for a key; the pressed key stands next to the old one in the pending
colour; `Enter` binds it in place of the old one, `Esc` keeps the old one. A key that
another action of the same section has lights that action and the line says it loses the
key; `Enter` moves it. A pad key, alone or with Shift, is not taken and the line says why:
the pad keys play the pads. `Backspace` takes the key of the action away after `Enter`; a
reset brings the shipped keys back, for the action or for all after a question. Every
change is saved in the owner's keymap at once and works at once.

#### Scenario: Rebind a key
- **WHEN** the owner picks "change scope" on the Keys tab, presses `Enter`, `Ctrl+G` and
  `Enter`
- **THEN** the row shows `Ctrl+G`, `Ctrl+G` steps the change scope on the pads screen,
  `Ctrl+A` no longer does, and the user keymap holds the change

#### Scenario: Esc keeps
- **WHEN** a key stands pending for an action and the owner presses `Esc`
- **THEN** the action keeps its old key

#### Scenario: A taken key
- **WHEN** the owner binds `M` to "change scope" on the pads screen, where `M` marks pads
- **THEN** the "marking" row lights and the line says it loses `M`; after `Enter` `M`
  steps the scope and marking has no key

#### Scenario: A pad key is refused
- **WHEN** the owner presses `Q` for an action
- **THEN** nothing is pending and the line says the pad keys play the pads

#### Scenario: Take a key away
- **WHEN** the owner presses `Backspace` and `Enter` on "change scope"
- **THEN** the action has no key and `Ctrl+A` does nothing

#### Scenario: Reset to shipped
- **WHEN** the owner resets "change scope"
- **THEN** it has `Ctrl+A` again, and the user keymap no longer names it
