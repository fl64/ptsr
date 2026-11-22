## 1. Shape the pad signal

- [x] Add pan and fine tune with smooth changes.
- [x] Check the pan law, stereo balance, effect sends and pitch limits.
- [x] Add hold, envelope curves and the velocity range.
- [x] Check timing, levels and the original linear response.
- [x] Render the pan, pitch, envelope and velocity cases.

## 2. Save the pad settings

- [x] Save the new defaults and each pad override.
- [x] Check older projects, A1 raw velocity, and the copy and undo paths.

## 3. Show and control the settings

- [x] Add the SOUND page and the Hold row in ENV.
- [x] Check keyboard and raw MIDI routes, fine steps with and without PAD BANK hold, the route table and the label glossary.
- [x] Check 80 by 26 and 120 by 40, compare the catalogue and check the callback budget.

## Validation

- Changed tests: 1793 passed.
- Screen catalogue: 42 tests passed. Compare 918 frames with 884 on the base.
- Release build and Clippy: pass.
- Steady playback at 32 voices: 5.78 percent p99 and 24.03 percent maximum.
- Timed envelopes and pan on 32 voices: 4.58 percent p99 and 17.68 percent maximum.
- Early checks found a stack overflow and old page routes. The engine allocates
  its pad table at startup. The current test run covers the updated routes.
- Physical controller and listening checks follow merge.
