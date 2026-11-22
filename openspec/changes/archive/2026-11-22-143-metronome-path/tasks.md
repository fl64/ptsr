## 1. Monitor path

- [x] Split the click from the master mix. Test timing, levels, the meter and Panic.
- [x] Test the callback budget against the base.

## 2. Record the pre-click mix

- [x] Keep JACK sources on the chosen output in the recording.
- [x] Test signal isolation, mono routing and source changes in an isolated audio graph.
- [x] Render the monitor and recording for the listening check.

## 3. Click control

- [x] Add the saved Audio setting, keyboard path and controller knob.
- [x] Check both screen sizes, input routes and the owner procedure.

## Verification

The fast run passed 1773 tests. The screen run passed 39 tests.
The screen comparison changed 96 of 884 frames. All changed frames show Settings.

The paired callback run used 32 voices at 48 kHz and 128 frames. Steady wall p99
was 5.94 percent on the base and 3.97 percent on the head. The head maximum was
16.57 percent. The first run had one dense playback block at 74.62 percent wall
time with thread CPU at or below 16.38 percent across the case. The paired run
stayed within the limit.

Physical controller and listening checks follow merge.
