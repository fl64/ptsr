# Proposal

## Why

The owner cannot start a beat clock or keep a metronome running. Patterns need this clock before they can play or record hits.
The owner requested autonomous pattern development and deferred separate sound-quality work.

## What Changes

- Add a 4/4 clock at 40–300 BPM, with 120 BPM as the default.
- Start and stop with Space on Pads, FX and Settings. Keep Space unchanged in Chop and Files.
- Change tempo with Ctrl+Up/Down by 1 BPM, or Ctrl+Shift+Up/Down by 0.1 BPM. Alt+M toggles the metronome.
- Make transport buttons, tempo buttons and an absolute tempo knob available through existing MIDI learn.
- Show tempo, play state, bar and beat. Save tempo and metronome preference with each project and scratch session.
- Keep existing pad sounds, mute groups, assignment defaults, effects and owner bindings.

## Capabilities

### New Capabilities

- `transport`: beat clock, play/stop, accented metronome and visible position.

### Modified Capabilities

- `keyboard`: new default transport controls and unchanged preview keys.
- `midi-learn`: transport actions and tempo parameter in GLOBAL.
- `projects`: transport settings, format migration and stopped project restoration.
- `screen-layout`: transport beside existing modes, compact labels before lower-priority badge removal.

## Impact

The engine gains transport commands and an audio-thread clock. Application input, MIDI learn, session files, help and title feedback gain transport support.
Saved format 4 adds tempo and metronome preference. Formats 1–3 open with 120 BPM and metronome off. Existing pad and chop migrations stay intact.
No new dependency is necessary.

The existing X-Session profile and learned bindings remain usable. SMC-PAD hold controls need later input/profile work and physical verification.
This change does not claim completion of changes 88–93a or physical-controller acceptance.

## Out of scope

Pattern playback (96), recording/count-in (97), step editor (98), half-time, MIDI clock, sound corrections, smoothing, limiter and sample-rate conversion.
Do not change existing callback tests or budget limits. Historical maximum-budget failures remain unresolved.
Do not reset controllers or turn the incomplete SMC-PAD template into a verified profile.

## How to check

Use the isolated owner procedure in [Live checks](../../../../docs/LIVE-CHECKS.md#95-transport-and-metronome).
Do not use an existing performance project for the first test. Saving writes format 4, which older binaries refuse.

1. Open Pads. Press Alt+M, then Space. Hear four beats per bar, with a stronger first beat.
2. Change tempo with Ctrl+Up/Down. Add Shift for 0.1 BPM. Check the tempo and running bar/beat.
3. Press Space to stop. Hit a pad before stopping: its one-shot tail must continue. Start again from bar 1, beat 1.
4. Open Chop and Files. Check that Space still controls their existing previews, not the beat clock.
5. In F12 MIDI, learn buttons for transport/metronome and a knob for tempo in GLOBAL. Check their actions and saved bindings.
6. Save and reopen two projects with different tempos. Each opens stopped with its own tempo and metronome preference.
7. Listen to `renders/95-transport.wav`: 120 BPM accents, a tempo change, stop/restart, then 172 BPM accents.

Keyboard and physical-controller verification are separate. Offline MIDI tests do not establish a physical-controller pass.
