# Design

## Input contract

A plain action string keeps its current press timing. A button with a `hold` action waits. A release before the threshold sends `tap` once. At the threshold, or before another control on the same port acts, it sends `hold` down once. Release after that sends `hold` up once and no tap. A toggle such as `metronome.toggle` acts on hold down. Its up event has no action. `record.mic` starts on hold down and stops on hold up.

The gesture key is port, channel, and control. Another control on the same port can start the hold even when it uses a different MIDI channel. This matters for devices whose pads and buttons use different channels. The other control keeps its own action, and the hold event precedes it. A release or duplicate value does not count as use of another control.

The MIDI receipt time determines whether a release is a tap. The worker checks due holds after queued messages. A repeated press of the same held control does not restart its timer or send another action. A disconnect ends active hold actions and clears pending taps without making a tap. An invalid shared input state does not start a new gesture. An existing up action can still end. If the device loses only a release message, the next press looks like a repeated press. The hold stays active until release or disconnect.

## Profile and saved bindings

The profile accepts an action string or `{ tap = "...", hold = "..." }` in a screen or edit binding. A map needs `hold`. It can omit `tap`. A string in an existing user config remains a press action. New user-config maps store tap and hold in the same control entry. Reading an old string does not rewrite the user file. Profile seeding copies both actions once. The program refuses unknown actions, including a future `layer.*` action. Its message names the profile file and control. Change 91 will add an SMC-PAD factory profile after the owner measures its Linux messages.

Use existing actions in test profiles: PLAY can use `transport.toggle` for tap and `metronome.toggle` for hold. RECORD can use `record.output` for tap and `record.mic` for hold. Any plain button acts on press. The plan's `play.context` example does not name an existing action. Use `transport.toggle` in this change. A dedicated STOP action and screen roles belong to change 92.

## MIDI learn and setting

On a button action row, J and L select tap or hold before the owner moves a control. Tap is the default. The pending line names the selected gesture and shows the action that would lose that slot. Enter saves only the selected slot. Esc keeps both slots. Clearing a row removes that action from its slot without removing a different action on the same control. Pad and bank runs keep their existing immediate press behavior.

F12 MIDI shows a `hold time` row. J and L change the time in 50 ms steps. The allowed range is 200 to 600 ms. The default is 300 ms. The program saves the value under `[midi_gestures]`. It applies the value to all ports. A bad saved value uses 300 ms with a warning. A changed value applies to gestures that start after the change.

## Edge cases

| Situation | Expected result | Status | Check |
| --- | --- | --- | --- |
| A tap release arrives near the threshold | Its MIDI receipt time decides tap or hold. | Specify now | Timed raw events |
| Two devices hold the same CC number | Each device keeps its own timer and release. | Specify now | Recorded two-port sequence |
| The owner uses a pad on channel 10 while a button on channel 1 is down | The button hold starts before the pad action. | Specify now | Raw MIDI profile and dispatch |
| A hold changes the current screen | Its up event ends the original hold action. | Specify now | Recorded screen switch |
| The device disconnects during microphone recording | The microphone receives up once. No tap runs. | Specify now | Recorded disconnect |
| A release is lost, then a new press arrives | The repeated press does not start a second action. Release or disconnect ends the hold. | Limit | Recorded repeated press |
| The owner presses a plain button | Its action runs immediately. | Preserve | Raw MIDI dispatch |
| The program loads a saved string binding | It runs on press. The program does not rewrite it. | Preserve | Config round trip |
| A future layer action is in a profile | The program refuses the binding with a message. | Specify now | Profile load error |

## Open questions

- If the owner holds a control and never sends a release or another press, should a maximum duration end it? Default: no. Disconnect ends it. A fixed maximum would stop a deliberate long recording.
- What action should a physical STOP button use on each screen? Default: leave its role to change 92 after the owner measures the profile. This change tests the immediate timing of a plain binding.

## Verification

Use a pure gesture tracker with supplied times. Feed recorded raw MIDI through a test profile and production dispatch for tap, threshold, chord, duplicate press, disconnect, two ports, and two channels. Check MIDI learn and config persistence through the application path. Compare keyboard behavior and check the F12 MIDI row at 80x26 and 120x40. This change does not change sound processing and needs no new WAV.
