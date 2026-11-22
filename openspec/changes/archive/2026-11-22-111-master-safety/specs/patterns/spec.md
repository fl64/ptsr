## MODIFIED Requirements

### Requirement: Pattern timing uses audio frames

Pattern onsets and scheduled releases SHALL occur within one output frame of their clock positions plus the fixed device delay. Timing SHALL remain independent of redraws and audio block lengths. Loop seams and callback boundaries SHALL NOT miss or double events. Empty processing SHALL NOT consume events, deadlines or musical time.

#### Scenario: Pattern timing is independent of callback size
- **WHEN** identical patterns render at 40, 120, 172 and 300 BPM with 1-frame, 128-frame, 1024-frame and irregular blocks
- **THEN** each onset and release stays within one output frame of its clock position plus the fixed device delay
- **AND** tick-zero hits and events near block or loop boundaries occur once

#### Scenario: Pattern empty blocks preserve its first event
- **WHEN** empty blocks occur before the first non-empty block or between playing blocks
- **THEN** only processed output frames advance the pattern and its release deadlines

### Requirement: Recorded timing follows accepted audio

Live recording SHALL timestamp manual input at its accepted audio-block start. Input timing uncertainty SHALL remain at most one actual audio buffer. Raw ticks SHALL round to the nearest integer, with ties later. Playback of a completed take SHALL retain the existing one-output-frame scheduling accuracy after accounting for the fixed device delay. Redraw timing remains independent.

#### Scenario: Recorded positions follow audio blocks rather than redraws
- **WHEN** identical accepted input renders with different UI delays and known audio-block boundaries
- **THEN** raw ticks match those audio boundaries
- **AND** later playback follows the raw or quantized ticks within one output frame after accounting for the fixed device delay
- **AND** the result does not claim sub-buffer live input accuracy

### Requirement: Fresh hits do not play twice

A manual hit SHALL sound after the fixed device delay without another same-lap trigger from its new row. Its first sequenced occurrence SHALL be in a later eligible loop. A quantized seam hit SHALL become eligible at the next loop start. Delayed publication SHALL NOT replay past occurrences. Recompiling settings SHALL NOT repeat an already played occurrence.

#### Scenario: A hit before the grid sounds once in its capture lap
- **WHEN** a manual hit occurs 20 ms before a grid line and recording applies 100 percent quantize
- **THEN** it sounds once after the fixed device delay, not again on that same-lap grid line
- **AND** its later loop occurrence plays on the grid within one output frame after accounting for the fixed device delay

#### Scenario: A recorded seam hit first repeats at the next loop start
- **WHEN** a late-loop manual hit quantizes to the loop end and wraps to zero
- **THEN** it sounds once manually and first repeats at the next eligible zero tick
- **AND** it neither repeats before that seam nor skips an extra complete loop

#### Scenario: Delayed capture publication does not produce a catch up burst
- **WHEN** UI delay postpones a pattern update beyond one or more eligible occurrences
- **THEN** elapsed occurrences stay missed and only future eligible occurrences play
- **AND** grid changes and additional capture updates do not repeat consumed occurrences

### Requirement: Playback plays saved ticks

Playback SHALL play each visible row at its saved tick plus the fixed device delay. It SHALL NOT quantize the rows again. Same-pad rows on one tick SHALL play once by the 96 collision order.

#### Scenario: A settings change does not move playback
- **WHEN** A1 plays saved rows and the owner changes Quantize and Strength through every value
- **THEN** the rendered onsets stay at the saved ticks within one output frame after accounting for the fixed device delay
- **AND** the rendered output has no clipping, non-finite sample or click
