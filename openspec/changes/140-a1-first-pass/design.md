# Design

## Decision

Show the next step in the label space to the left of the step numbers. This space keeps its width and does not take rows from the table. The help band keeps its navigation and edit controls.

The [Roland SP-404MK2 manual](https://static.roland.com/manuals/sp-404mk2_reference_v300/eng/62589587.html) and the [Akai MPC Sample tutorial](https://www.akaipro.com/guides/mpc-sample/tutorial.htm#recording-a-sequence) show the separation between a live pass and listening to its loop. PTSR uses its own current transport and record controls.

The overview reads accepted audio state. A pending start or finish shows a wait instruction. Help, questions, step edit, erasure, and other active work keep their own directions. Save and audio warnings keep their rows.

## Open questions

- First action in an empty project: default to live pad recording. The owner authorized autonomous work. A step-first creation flow can be reviewed separately.

## Cases

| Situation | Expected cue | Status |
| --- | --- | --- |
| A1 is absent or has no visible hits | Effective record control | Planned |
| The user presses record while a request is pending | Wait for the current request | Planned |
| A count-in runs | Cancel control | Planned |
| Pads record hits | Play pads and finish the pass | Planned |
| The loop runs or pauses | Stop or play control | Planned |
| Audio is unavailable | Wait or show that A1 needs audio | Planned |
| A binding changes or a higher priority mode opens | Show the effective control or let that mode take priority | Planned |
