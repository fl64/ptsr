## Decision

Split the automatic click and A1 count-in from the master mix in the engine. Measure and record the master before the click. Add the click at the device output. Put a future master limiter before this addition. Keep the click buffer available for a later cue output.

With JACK, read the source connections of the selected playback ports. Copy other sources to the recording inputs while a recording runs. Mix these inputs with the sampler master before the click. Reconcile these links when the output or its sources change. Do connection work outside the audio callback. Do not change any source link to the output.

Keep the current cpal recording support and keep mic capture separate. If JACK cannot set up the recording links, report the failure. If it loses the selected output during a take, retain the recorded part and report the stop.

The probe used a separate PipeWire server with a virtual output. An outside source, the sampler master probe and the cue probe summed to 0.875 at its monitor. The upstream record tap measured 0.375. The result matched with both a JACK source and a native PipeWire source from pw-play.

## Cases

| Case | Expected result | Status |
| --- | --- | --- |
| Change the click level while playing | Sound level changes without a step. Beat position stays put. | Planned |
| Use the click with an empty project | The output carries the click. The master meter is at zero. | Planned |
| Start or stop an outside source during a take | The capture follows its connection to the selected output. | Planned |
| Lose the selected output during a take | Keep the recorded part and show the reason. | Planned |
| Record on a mono output | Keep the selected output mix without adding the same outside source twice. | Planned |
| Start an A1 count-in or use Panic | The count-in goes to the device. Panic still stops the device sound. | Planned |

## Open questions

- Default click level: use 100 percent of the current click at unity master gain.
- Settings scope: store the level in Audio settings for all projects.
