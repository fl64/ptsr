# patterns Delta

## Purpose

Play a saved pad rhythm through the project beat clock. Keep raw timing, exact loop boundaries and manual playing independent of interface redraws.

## ADDED Requirements

### Requirement: One looped pattern

The system SHALL support zero patterns or one pattern named `A1`, with 1–64 bars of 4/4. A bar SHALL contain 3840 ticks. Transport start SHALL play from tick zero and repeat the pattern. Repeated explicit start SHALL NOT restart it. Without a pattern, transport and metronome SHALL retain their existing behavior.

#### Scenario: Pattern repeats from its first hit
- **WHEN** a two-bar A1 pattern runs for four loops at 120 BPM
- **THEN** every hit plays once per loop, including its tick-zero hit
- **AND** repeated explicit start does not restart the loop

#### Scenario: Empty pattern does not stop the beat clock
- **WHEN** transport runs with no pattern or a pattern with no hits
- **THEN** no sequenced pad sounds and the clock and enabled metronome continue

### Requirement: Raw pattern hits

Each raw hit SHALL contain an integer tick from 0–245759, absolute pad from 0–127, velocity from 1–127 and gate length from 0–245760 ticks. A pattern SHALL contain at most 4096 raw hits. Playback velocity SHALL equal velocity divided by 127. Changing the open bank SHALL NOT change an event's pad.

#### Scenario: Pattern addresses do not follow the open bank
- **WHEN** hits address pads 0 and 127 while the owner changes the visible bank
- **THEN** the same bank 1 and bank 8 sounds play with their saved velocities

### Requirement: Playback quantize preserves raw timing

Quantize SHALL support Off, 1/4, 1/8, 1/16, 1/32, 1/64, 1/8T, 1/16T and 1/32T. Default SHALL be 1/16 at 100 percent strength. Strength SHALL range from 0–100 in steps of 10. Playback SHALL move each hit toward its nearest grid line by that percentage. Off SHALL preserve raw timing. Saved raw hits SHALL remain unchanged.

#### Scenario: Pattern quantize and strength preserve the raw take
- **WHEN** the same off-grid raw hits compile with every supported grid at strengths 0, 50 and 100
- **THEN** playback uses the corresponding grid movement and raw ticks stay unchanged
- **AND** Off keeps the raw timing at every strength

### Requirement: Quantize rounding and collisions

The nearest grid midpoint SHALL choose the later line. Fractional output ticks SHALL round to the nearest integer, with half choosing the later tick. Same-pad hits at the same compiled tick SHALL play once. The higher velocity SHALL win, then the longer gate length, then the earlier raw row. Raw rows SHALL NOT merge.

#### Scenario: Pattern midpoint and collision rules are deterministic
- **WHEN** raw hits lie at exact grid midpoints or compile onto the same pad and tick
- **THEN** rounding follows the later-line rule and each collision uses the specified winner
- **AND** different pads on that tick retain their separate hits

### Requirement: Hidden hits survive loop shortening

Playback SHALL exclude raw hits at or beyond the current loop length before quantization. These raw hits SHALL remain saved and return when the loop grows. Only included hits that quantize to the loop end SHALL wrap to tick zero.

#### Scenario: Pattern shortening cannot wrap hidden hits into the start
- **WHEN** a two-bar pattern shortens to one bar with hits before, on and after tick 3840
- **THEN** hits on and after tick 3840 stay saved but do not play or quantize into the start
- **AND** growing back restores them, while an included hit quantized to the seam plays at tick zero once

### Requirement: Pattern timing uses audio frames

Pattern onsets and scheduled releases SHALL occur within one output frame of their clock positions. Timing SHALL remain independent of redraws and audio block lengths. Loop seams and callback boundaries SHALL NOT miss or double events. Empty processing SHALL NOT consume events, deadlines or musical time.

#### Scenario: Pattern timing is independent of callback size
- **WHEN** identical patterns render at 40, 120, 172 and 300 BPM with 1-frame, 128-frame, 1024-frame and irregular blocks
- **THEN** each onset and release stays within one output frame of its clock position
- **AND** tick-zero hits and events near block or loop boundaries occur once

#### Scenario: Pattern empty blocks preserve its first event
- **WHEN** empty blocks occur before the first non-empty block or between playing blocks
- **THEN** only processed output frames advance the pattern and its release deadlines

### Requirement: Sequenced hits preserve pad sound rules

Sequenced hits SHALL use the current pad mode, volume, pitch, source bounds, envelope, filter, drive, sends and mute group. Same-pad retriggers and group cuts SHALL use their existing short fade. Simultaneous sequenced hits SHALL run in ascending absolute-pad order after block-start manual commands. Empty, missing and loading pads SHALL be silent without later replay.

#### Scenario: Pattern sound matches ordinary pad playing
- **WHEN** a pad renders through manual commands and equivalent pattern events with Pitch Gate Off, matching velocity, settings and gate releases
- **THEN** output differs by at most 0.000001 per sample
- **AND** mute groups cut the same sounds while unrelated groups and ungrouped pads ring on

#### Scenario: Pattern cuts do not start before the event
- **WHEN** a sequenced same-pad retrigger or mute-group hit falls inside an audio block
- **THEN** the previous voice continues unchanged until that event frame and starts its existing cut there

#### Scenario: Simultaneous pattern hits use a stable order
- **WHEN** a manual block-start hit and simultaneous pattern hits share a pad or mute group
- **THEN** manual commands retain accepted order and the pattern hits follow in ascending absolute-pad order
- **AND** the last applicable trigger determines the surviving voice under existing cut rules

#### Scenario: Unavailable pattern pads never replay missed hits
- **WHEN** an event addresses an empty, missing or loading pad and that pad becomes available later
- **THEN** the earlier hit stays silent and only future events can play it

### Requirement: Sequenced gate lengths

A gate-mode event SHALL release its own note after its saved length through the pad's release envelope. Zero-length gate events SHALL be silent. One-shot events SHALL ignore gate length. Gate deadlines SHALL retain their musical duration across loop seams and tempo changes. Tempo changes SHALL NOT reset the clock or repeat events.

#### Scenario: Pattern gate length crosses a loop seam safely
- **WHEN** a gate note near the loop end has a positive length extending into the next loop
- **THEN** only that note releases at its saved musical deadline through its release envelope
- **AND** a zero-length gate note is silent while a one-shot with zero length plays normally

#### Scenario: Pattern tempo changes preserve pending musical deadlines
- **WHEN** tempo changes while a pattern and gate notes run
- **THEN** future hits and releases follow the new tempo without resetting position or losing metronome phase

### Requirement: Automatic releases retain note ownership

A scheduled release SHALL affect only the note that created it. Retrigger, voice stealing, source replacement, pad swap, stop and project replacement SHALL NOT leave a deadline able to release another note. Manual release SHALL NOT release a newer sequenced note. Existing manual-to-manual release behavior SHALL remain unchanged.

#### Scenario: Old pattern release cannot cut a new manual hit
- **WHEN** a sequenced gate note or Pitch Gate note is followed by a manual hit on that pad before its old deadline
- **THEN** the old deadline cannot release or cut the manual voice

#### Scenario: Manual release cannot cut a newer pattern note
- **WHEN** a held manual pad is retriggered by the pattern before the manual control is released
- **THEN** the manual release leaves the pattern note to its own deadline

#### Scenario: Pattern deadlines cannot follow a reused voice slot
- **WHEN** notes retrigger, are stolen, change source or end before their deadlines and their slots are reused
- **THEN** no stale deadline releases any replacement voice

### Requirement: Pattern stop and restart

Stop SHALL prevent new pattern hits, release sequenced gate notes and cut active Pitch Gate notes through their respective release paths. Other one-shot tails, manual voices, previews and shared effects SHALL retain their existing behavior. Stop SHALL clear pending pattern deadlines and reset position. Restart SHALL start with one tick-zero event.

#### Scenario: Pattern stop preserves unrelated tails and clears deadlines
- **WHEN** Stop occurs with sequenced gates, Pitch Gate notes, ordinary one-shots, manual voices, previews and effect tails
- **THEN** only the sequenced gates and Pitch Gate notes receive automatic releases or cuts
- **AND** later restart has no old deadlines and plays its first event once

### Requirement: Same-project pattern replacement

Replacement SHALL take effect atomically at an audio block boundary, without resetting transport or metronome phase. It SHALL release old sequenced gate and Pitch Gate notes. Events whose nearest frame precedes that boundary SHALL NOT replay. Events at or after it SHALL follow the replacement. Removing a pattern SHALL leave transport running.

#### Scenario: Pattern replacement keeps phase without a duplicate hit
- **WHEN** a replacement adds events before, on and after the next block boundary while transport runs
- **THEN** past events do not replay and each applicable new event sounds once
- **AND** old gate deadlines cannot affect new notes and removing the pattern leaves the clock running

### Requirement: Bounded pattern playback under resource pressure

Playback SHALL NOT allocate, deallocate, lock, log, access files or wait in the audio callback. Retirement pressure SHALL defer whole replacements while the old pattern continues. If automatic voice retirement cannot proceed, the hit SHALL be omitted and counted, not delayed. The UI SHALL warn about dropped pattern hits. Normal dense playback SHALL drop no hit.

#### Scenario: Pattern retirement pressure does not block or free in the callback
- **WHEN** retirement is full during playback, replacement, obsolete-command disposal or project transition
- **THEN** references remain owned until safe retirement without callback allocation, deallocation or waiting
- **AND** deferred replacements stay whole, omitted automatic hits increment the warning count, and recovery produces no late-hit burst

### Requirement: Bounded automatic release storage

Automatic release storage SHALL be bounded. Stale entries SHALL expire without affecting voices. When capacity is exhausted, the earliest pending deadline SHALL release only its own note before a new entry is stored. Equal deadlines SHALL choose the oldest note. The callback SHALL NOT allocate storage or use a pad-wide release for this case.

#### Scenario: Pattern release capacity cannot release another owner
- **WHEN** the fixed release capacity is reached and another deadline needs storage
- **THEN** only the oldest applicable pending note releases and the new deadline remains tracked
- **AND** manual voices and reused note identities remain unaffected
