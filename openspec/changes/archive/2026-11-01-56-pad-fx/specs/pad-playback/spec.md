## ADDED Requirements

### Requirement: Pad drive
The system SHALL give every pad a drive from 0 to 100 %: turning it up adds grit and keeps
the loudness about the same. A new pad and a pad of an older project have the drive at 0;
at 0 the pad sounds exactly as without it. A change on a sounding pad glides, no click.
The drive follows the change scope and is saved with the pad in the project.

#### Scenario: Off is untouched
- **WHEN** a pad has the drive and both sends at 0 and is hit
- **THEN** the render is the same, sample for sample, as before the effects existed

#### Scenario: Grit, not volume
- **WHEN** the test tone pad has the drive at 80 % and is hit
- **THEN** the render has more energy above the third harmonic than at 0 %, its peak stays
  within 3 dB of the peak at 0 %; no clipping, no NaN

### Requirement: Shared effect slots
The system SHALL have two effect slots, FX 1 and FX 2, shared by all pads of the project.
Every pad sends a share of its sound after the drive to each slot, 0 to 100 %, 0 on a new
pad and on a pad of an older project; the slot output is added to the pad's own sound.
A slot holds one effect of the list: reverb (size, tone) or delay (time 10–2000 ms,
feedback 0–90 %). A new project has a reverb in FX 1 and a delay in FX 2. A slot sounds on
after the pads that fed it stop, until its tail dies out. Every parameter change glides,
no click; a type change fades the old tail out and starts the new effect empty, and the
slot keeps the parameters of every type, so a type turned back finds them as they were.
The slots are saved in the project. The audio callback with 32 voices and both slots
stays within 50 % of the buffer budget (128 samples at 48 kHz).

#### Scenario: The echo
- **WHEN** a short pad sends 100 % to a delay with the time at 250 ms and the feedback at
  50 % and is hit once
- **THEN** the render has an echo 250 ms after the hit, within 1 sample, and the next echo
  250 ms later about 6 dB quieter; no clipping, no NaN

#### Scenario: The tail outlives the pad
- **WHEN** a one-shot pad of 100 ms sends 60 % to a reverb and is hit
- **THEN** the render has sound after the pad's end, and it dies down to silence within a
  few seconds

#### Scenario: Turn the time while it echoes
- **WHEN** the delay echoes and its time is turned from 250 to 400 ms in one step
- **THEN** no step between two neighbouring samples is larger than the sound's own

#### Scenario: Two pads, one room
- **WHEN** pads Q and W send to FX 1 with a reverb and the size is turned up on the FX1*
  page of pad Q
- **THEN** the tail of a hit on W is longer too

#### Scenario: A new type in the slot
- **WHEN** FX 2 sounds with a delay tail and its type is turned to reverb, then back to delay
- **THEN** the delay tail fades out without a click, and the delay has the time and the
  feedback it had before

#### Scenario: Two reverbs
- **WHEN** FX 1 and FX 2 both hold a reverb, pad Q sends to FX 1 with size 20 % and pad W
  to FX 2 with size 90 %
- **THEN** the tail of W is longer than the tail of Q

#### Scenario: Saved and opened
- **WHEN** pad W has the drive at 30 % and send 1 at 40 %, FX 2 holds a reverb with size
  70 %, the project is saved and opened again
- **THEN** all of it is as before and sounds as before

#### Scenario: An older project
- **WHEN** a project saved before the effects existed is opened
- **THEN** every pad has the drive and both sends at 0, FX 1 holds a reverb and FX 2 a delay

#### Scenario: Marked pads together
- **WHEN** pads Q and W are marked and the owner turns send 1 on Q to 30 %
- **THEN** send 1 of W is 30 % too, and the slot pages change no pad

## MODIFIED Requirements

### Requirement: The pad panel in blocks
The system SHALL group the parameter rows of the pad panel by page, in the page order
TONE, ENV, FILTER, SOURCE, FX: every block stands under a thin line with the page name,
the line of the page the edit is on is lit. While the edit is on a shared slot's page,
that slot's block (`FX 1 * shared`, `FX 2 * shared`) stands in place of the FX block. The edit walks the pages in the page order with `Tab` and `I` `K`, and
the frame title names them in it.

#### Scenario: Blocks
- **WHEN** the pad edit is open
- **THEN** the panel shows the lines TONE, ENV, FILTER, SOURCE, FX in this order, each over its rows

#### Scenario: SOURCE last
- **WHEN** the edit is on the reso of the FILTER page and the owner presses `K`
- **THEN** the source is picked, on the SOURCE page; `K` after the end of SOURCE picks the
  drive on FX, and `K` after the last parameter of FX2* goes back to the volume
