## 1. Limit the master

- [x] Set the source and monitor headroom rules.
- [x] Implement the stereo limiter with the measured delay.
- [x] Check the sample ceiling, oversampled peaks, unity level and callback budget.

## 2. Preserve timing and stop

- [x] Align the monitor cue and the mastered sound.
- [x] Check recorded ticks, pattern onsets, release deadlines and Panic.
- [x] Render normal and loud passages for the owner.

## 3. Control FX returns

- [x] Save each return level and mute state with compatible defaults.
- [x] Add the keyboard and controller paths and effects display.
- [x] Check both terminal sizes, clicks and slot isolation.

## Verification

The final changed run passed 1781 tests. The screen run passed 41 tests.
The comparison covered 884 frames. FX changed in 108 frames, and MIDI settings changed in six.

At 32 voices and 128 frames at 48 kHz, steady playback wall p99 was 6.72 percent
on base and 5.86 percent on head. The head wall maximum was 11.78 percent.
A base block reached 90.89 percent wall time. The head stayed within 50 percent,
and neither run dropped dense pattern hits.

The first fast runs found a label length error, an incomplete route test, a
changed MIDI learn order, and compact FX layout regressions. The final run passed.
Physical controller and listening checks follow merge.
