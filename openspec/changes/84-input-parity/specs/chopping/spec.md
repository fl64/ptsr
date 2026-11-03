## ADDED Requirements

### Requirement: Pad playback line on Chop

While Chop is open in listen or slice edit and a pad plays a slice of the open chop,
the system SHALL show the existing thin green playback line at that voice's actual
position in the sample. Keyboard pads, raw MIDI notes and explicitly learned pad
buttons SHALL give the same display behavior. The position SHALL follow the real
pad playback, including pitch and retrigger, and SHALL disappear when that voice ends.
The line SHALL use the current waveform view in Braille and picture modes. It SHALL
NOT show another sample's position or keep an old project's position.

This addition SHALL change only playback position display. It SHALL NOT add another
pad hit or preview, change the sound, move the owner's cursor as playback advances,
or change the existing hit-driven slice selection. Space, M, transport, editing,
preview and assignment behavior SHALL remain unchanged. In particular, the pad's
new display position SHALL NOT replace the preview position used by those commands.
A silent, blocked or queue-rejected hit SHALL NOT create a new playback line.

#### Scenario: Keyboard pad shows the Chop playback line
- **WHEN** a linked slice is open on Chop in listen or slice edit, the owner hits its
  keyboard pad and the real audio callback advances that pad voice
- **THEN** the green line follows that voice's frame position in the waveform
- **AND** the cursor keeps the existing hit selection but does not follow playback

#### Scenario: MIDI pad shows the same Chop playback line
- **WHEN** the same linked slice is hit from a raw MIDI note or explicitly learned pad
  button through the real dispatch and audio queues
- **THEN** the green line follows the same sounding position as the keyboard path
- **AND** no extra pad hit or preview is started to supply that position

#### Scenario: Chop pad line follows pitch and retrigger
- **WHEN** a linked slice pad plays with changed pitch or is hit again on Chop
- **THEN** the green line follows the voice's real sample position and returns to
  that slice's start on the new hit, not elapsed UI time or a bare preview's speed

#### Scenario: Chop pad line ends with its voice
- **WHEN** a displayed pad voice reaches its end, finishes its gate release or is stopped
- **THEN** that voice's green line disappears without starting or stopping another voice

#### Scenario: Chop pad line keeps transport and edits unchanged
- **WHEN** a linked pad sounds on Chop, its new green line is visible and the owner
  moves the cursor, presses M or uses Space
- **THEN** the cursor and commands keep their existing behavior, independent of the
  new pad position display
- **AND** Space preview, bare slice previews and assignment previews keep their existing
  playback line behavior

#### Scenario: Chop pad line rejects unrelated and silent hits
- **WHEN** a pad plays another sample, the project changes, a question or learn blocks
  a new hit, or the audio queue rejects a new hit
- **THEN** no line is created from that unrelated, old-project, blocked or rejected hit
- **AND** an already sounding matching voice keeps its valid position until it ends
