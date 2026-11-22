# Master safety timing decision

The plan calls for a master limiter with one to two milliseconds of look-ahead.
Current specifications require the click, pattern onsets and scheduled releases within
one output frame of the clock. A delay on the master alone moves the pad away from the click.

Use 1.5 ms of fixed device latency. At 48 kHz this is 72 frames. Delay the cue by the
same number of frames and add it after the master limiter. Keep the pattern clock,
recorded ticks and quantize on the original grid. Measure device onsets and releases
relative to that grid plus the fixed latency. All existing relative timing stays within
one frame. Update the transport, pattern and pitch gate requirements before this code.

Preserve the five-millisecond Panic deadline measured at the device. Account for
the pending frames once, and then fade the old source to zero before the deadline.
Check a new pad hit, a project switch and recorded pattern data during this fade.

## Bench and listening cases

- A unity-gain low-level mix must keep its level after time alignment.
- A loud mix uses 32 voices, drive, reverb and delay. The master must stay below
  minus 0.3 dBFS. Measure true peaks with a separate 4x reference.
- Test the final device mix with the click and a limited master. The master recording
  still excludes the click. Set a cue headroom rule before coding the limiter.
- Test steady and loud mixes at 44.1, 48 and 96 kHz. Include 1-frame, 128-frame,
  1024-frame and irregular blocks. Report callback load in the full stress case.
- Move each FX return between 0 and 100 percent and mute one slot. Check the
  independent return and a clean transition in the rendered waveform.
# Master and monitor headroom

The master sample ceiling is minus 0.3 dBFS. A click at unity setting has a sample
peak of 0.5. If these peaks coincide, the device mix can reach 1.466 in a float
pipeline. The monitor mix needs its own headroom.

Use a gain for the cue only when the combined signal needs protection. Leave the
limited master and the recorded signal at their original gain. Compute the safe
cue gain before the device delay ends. Smooth its release. Test the click accents
with spare headroom and check a loud mix where the cue needs attenuation.

Use a 4x reference that can see peaks between samples. Use a 12 kHz sine at 48 kHz with a 45 degree phase.
The sample peak is minus 0.3 dBFS. The 4x peak is plus 2.71 dBTP. A sample ceiling alone does not pass this test.

Preserve unity on the master when the mix peaks below minus 6 dBFS. Treat master
0 dB as a gain of one. Check an ordinary 32-voice mix and a loud mix with both FX
returns. Keep the default return at 100 percent for an existing project.
# Shared FX return controls

A project keeps a return level and mute state for each shared FX slot. An older
project uses 100 percent and unmuted for both slots. Smooth changes in the wet
path. Keep the dry pad signal and the other slot at the same level.

Use the current slot on the FX screen. The keyboard can select Return with the
same keys that select the other parameters. Left and Right change its level.
Use M to toggle the current return mute.

The SMC-PAD uses E5 for the return of the current slot. On the FX panel, P13
toggles mute for that slot. Use the existing FX slot control to switch slots.
The FX frame and controller help show the slot, the return level and mute.
Check the labels and controls at 80 by 26 and 120 by 40.

A limiter indicator names gain reduction above 1 dB. Keep it visible long enough
to read in the shared output meter. Do not cover a warning or a question.

## Audio boundary cases

| Case | Expected result | Check |
| --- | --- | --- |
| A quiet block or an empty callback | Quiet samples keep their level. Empty callbacks leave the delay unchanged. | Engine tests |
| A loud phase between samples | Sample ceiling and the independent 4x reference stay within the limit. | Engine render |
| Panic and then a new hit | The old output ends within five milliseconds. The new hit can play. | Callback test |
| A project switch at one frame per callback | Old frames drain before the new project is ready. | Callback test |
| Mute an active slot, then undo | The other return and dry mix stay at the same level. The old setting returns. | Engine and input tests |
| Open a project saved before the new controls | Each level starts at 100 percent, with both returns unmuted. | Project reload test |

Both device backends enable the same engine output stage before their callbacks start.
The engine can also render the mix before the device stage for scheduling tests.
Tests for this change and the callback benchmark enable the device stage.
The LIMIT mark holds for 400 ms after gain reduction exceeds 1 dB.
