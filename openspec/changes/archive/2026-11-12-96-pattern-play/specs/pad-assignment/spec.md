# pad-assignment Delta

## MODIFIED Requirements

### Requirement: Swapping pads

The system SHALL let the owner take the selected pad "in hand" with Ctrl+X and swap it with the next pad hit. Sample, bounds, mode, volume, pitch, envelope, filter, drive, sends, mute group and Pitch Gate SHALL move together. Every raw pattern hit on either pad SHALL exchange its absolute pad address with the other. Esc SHALL cancel.

#### Scenario: Swap two pads
- **WHEN** pad Q is selected, the owner presses Ctrl+X and then hits pad V
- **THEN** Q now plays what V played and V plays what Q played, with their modes and volumes
- **AND** the session saves

#### Scenario: Every setting moves
- **WHEN** pad Q has a low-pass filter, pitch +3 and a drive, pad V has none of them, and the owner swaps them
- **THEN** pad V has the low-pass filter, pitch +3 and the drive, sounds with them, and pad Q has none of them

#### Scenario: Move to an empty pad
- **WHEN** the pad in hand is put on an empty pad
- **THEN** the sample moves there and the first pad becomes empty

#### Scenario: Swap while loading
- **WHEN** one of the two pads is still loading a file
- **THEN** the pads are not swapped and the status says to wait until loading finishes

#### Scenario: Pattern hits follow swapped sounds across banks
- **WHEN** keyboard or raw MIDI swaps two pads in different banks with different sounds, Pitch Gate values and pattern hits
- **THEN** all raw hit addresses for those pads exchange, including hidden hits outside the current loop
- **AND** timing, velocity, gate length and the heard rhythm keep their sounds, while unrelated hits stay unchanged

## ADDED Requirements

### Requirement: Pattern pad swaps are atomic and undoable

A successful swap SHALL apply pad state and pattern addresses as one transaction, including during playback and queue pressure. One undo SHALL restore both together. Cancelled, self-targeted or refused swaps SHALL change neither and add no undo step. Save and reopen SHALL preserve successful swaps. Old note releases SHALL NOT target swapped replacement notes. Voices and deadlines on unrelated pads SHALL remain unchanged.

#### Scenario: Playing pattern swap and undo cannot split sound from hits
- **WHEN** a playing swap and its undo reach a full command or retirement queue through keyboard or raw MIDI routes
- **THEN** each accepted audio update applies the matching pad states and pattern addresses together
- **AND** one undo restores both, without a stale release cutting another voice or changing an unrelated gate note's deadline

#### Scenario: Cancelled pattern swaps leave raw hits unchanged
- **WHEN** the owner cancels a swap, targets its source, or receives a loading refusal
- **THEN** pad snapshots, raw hits, pending work and undo remain unchanged

#### Scenario: Pattern swap remains after save and reopen
- **WHEN** a successful cross-bank pad swap saves in a project or scratch session and that work reopens
- **THEN** both sounds, their Pitch Gate settings and every exchanged raw hit address remain as saved

### Requirement: Copy and replacement preserve pattern addresses

Pad copy SHALL copy Pitch Gate with the existing sound snapshot, but SHALL NOT copy or redirect pattern hits. File assignment and clear SHALL retain existing hit addresses. An empty target SHALL remain silent until it has sound. Undo SHALL restore the pad without rewriting its hits.

#### Scenario: Pattern addresses stay during copy clear and file replacement
- **WHEN** the owner copies, clears, assigns a file or undoes those operations on a pad addressed by A1
- **THEN** raw pattern hits stay unchanged and future events use that pad's resulting sound
- **AND** copy takes Pitch Gate from pickup, while a cleared pad's events are silent
