## Decisions

Use the recording state to select the STOP sequence. Keep work-screen preview
controls for sessions without an output recording. A visible question keeps its answer.

A playing preview consumes one STOP. A pending Files preview does not consume a
separate press. Invalidate its load before pausing A1 or saving the recording.

Permit STOP during Assign when output recording runs. Keep the plan and the
usual controls for leaving Assign.

Keep one save request for the active take if the next STOP arrives before A1 pauses.
Save the take when the audio thread reports the pause. Clear the request when the take ends.

## Edge cases

- Preview, A1 and recording together. Test the three presses in order.
- A1 and recording without a preview. Test Files, Chop and Assign.
- A pending Files preview while A1 is idle. Deliver the load after STOP.
- Pause feedback arrives before the next STOP. Check that saving preserves the
  paused position.
- Two STOP presses arrive before pause feedback. Test that both the keyboard and
  SMC-PAD save the take at the paused position.
- No active output recording. Keep the existing preview and transport tests.

## Open questions

None.
