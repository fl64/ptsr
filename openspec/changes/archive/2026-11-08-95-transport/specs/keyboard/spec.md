# Keyboard delta

## ADDED Requirements

### Requirement: Transport keyboard controls

Shipped Space SHALL toggle transport on Pads, FX and Settings through `transport.toggle`. Chop and Files Space SHALL keep preview playback. Ctrl+Up/Down SHALL change tempo by 1 BPM. Ctrl+Shift+Up/Down SHALL change it by 0.1 BPM. Alt+M SHALL toggle the metronome. These new controls SHALL work in English and Russian layouts. Commands SHALL NOT occupy plain or Shift pad keys.

#### Scenario: Space controls transport without replacing previews
- **WHEN** the owner presses Space on Pads, FX or Settings and then on Chop and Files
- **THEN** the first three screens toggle transport and the preview screens retain their previous Space actions

#### Scenario: Tempo and metronome keys work in both layouts
- **WHEN** keyboard events use Ctrl+Up/Down, Ctrl+Shift+Up/Down and Alt+M in either supported layout and input protocol
- **THEN** tempo changes by the specified amount or the metronome toggles once
- **AND** tempo holds at 40.0 and 300.0 BPM without wrapping

#### Scenario: Holding Space does not repeatedly toggle transport
- **WHEN** the owner presses Space, holds it through keyboard repeats and releases it
- **THEN** transport toggles only once

### Requirement: Transport keys preserve owner assignments

Existing owner keymap entries and `none` SHALL retain priority. The old `transport.play_stop` action SHALL keep its Chop preview meaning. `metronome.click` SHALL remain available without a new shipped key. New defaults hidden by owner entries SHALL use the existing startup warning queue without rewriting the keymap.

#### Scenario: Previous click and preview bindings retain their meaning
- **WHEN** an owner keymap binds Space to `metronome.click` or binds another key to `transport.play_stop`
- **THEN** those bindings keep the test click and Chop preview behavior without becoming general transport toggles

#### Scenario: Owner overrides hide new transport defaults with a warning
- **WHEN** owner entries change or remove the shipped Space, tempo or metronome bindings
- **THEN** their entries remain effective and startup warnings name the hidden defaults once per chord and affected screen

### Requirement: Transport help and input priority

Bottom hints and F1 SHALL name effective transport, tempo and metronome controls where applicable. The edit band SHALL keep its EDIT-only layout, with transport keys in F1. Text inputs, questions, pickers, screen learn and MIDI learn SHALL keep their existing input priorities. Selecting a transport command in screen learn SHALL NOT execute it.

#### Scenario: Transport input respects existing overlays
- **WHEN** transport keys arrive while text, a question, picker or learning mode owns input
- **THEN** that mode retains its existing behavior without starting transport or changing tempo through the underlying screen
- **AND** screen learn can select the effective transport command without executing it

#### Scenario: Help follows effective transport bindings
- **WHEN** shipped or owner transport keys are effective on the current screen
- **THEN** the bottom hints and F1 name those controls without replacing the screen's preview hints
