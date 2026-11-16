# Design

## Input model

A profile control in `[controls.knobs]` can use `type = "relative"` and `encoding = "offset64"`, `"twos"` or `"signbit"`. Other controls remain absolute unless their profile says otherwise. A relative control must name a parameter or one of the five list targets below. The loader rejects an unknown encoding or an incompatible binding with the profile file and control name. A bad profile falls back to the default profile. A user-learned action still overrides a profile action. The profile retains the physical control type after the program copies its roles into the user config. An unnamed learned CC remains absolute.

The decoder reads one seven-bit CC value. `offset64` uses `value - 64`: 65 means +1, 63 means -1, and 64 means zero. `twos` uses 1–63 as positive, 65–127 as `value - 128` and 0 or 64 as zero. `signbit` uses 1–63 as positive, 65–127 as `64 - value` and 0 or 64 as zero. Zero steps create no UI action and do not start a pending button hold.

`Resolved::Step` contains the named control, `StepTarget` and signed steps. The MIDI worker adds the port and original receipt time to the UI event. Relative steps do not use `jump`, `pickup` or `scale`. Absolute controls retain their current parameter path. A relative CC is movement even when its byte is zero. A neutral byte does not acknowledge a notice or promote a pending hold.

## Speed and fine control

Count detents for each port, channel and CC in the last 40 ms, including the current message. A count of 1, 2, 3 or at least 4 gives a multiplier of 1, 2, 4 or 8. Keep the direction of the current message. Multiply only its decoded steps. Use receipt times, not UI delivery times. Clear the speed history on disconnect. The first message after a gap greater than 40 ms has multiplier 1.

`knob.fine` is a global button action. A press sets fine mode for its port. Its original release clears fine mode, even after a screen or modal change. Disconnect and invalid-state release clear it. While fine mode is on, the multiplier stays 1. Fine mode does not change a parameter that has no smaller stored unit. No SMC-PAD button gets this role in change 90. A slow single detent uses the keyboard tap step without the modifier.

## Step table

A relative turn applies one logical UI event. The app groups all steps from that event into one Undo step. It uses the existing parameter limits and scope. The table below defines one unaccelerated detent. The fine button uses the right column where it differs.

| Target | Normal detent | Fine detent |
| --- | --- | --- |
| Pad or master volume | 1 dB | 1 dB |
| Pad pitch | 1 semitone | 1 semitone |
| Attack, decay or release | 1 ms | 1 ms |
| Pad filter kind, source or FX kind | one option | one option |
| Pad cutoff | 1 semitone | 1 semitone |
| Pad resonance, drive, sends or FX percentages | 1 % | 1 % |
| Delay time | 10 ms | 10 ms |
| Pad start or end | 1 ms | 1 ms |
| Tempo | 1 BPM | 0.1 BPM |
| Pattern bars or quantize | one option | one option |
| Pattern strength | 10 % | 10 % |
| Slice start or end, cursor or region edge | one visible fine step | one visible fine step |
| Chop zoom | one keyboard zoom step | one keyboard zoom step |
| Chop threshold | 1 dB | 1 dB |
| Edit page, bank or list row | one item | one item |

`edit.param_1..3` resolves to the active page parameter. `pad.fx1_*` and `pad.fx2_*` resolve to the corresponding shared FX parameter. A slot with no parameter does nothing. Open pattern controls keep their existing ownership of the three slot knobs. Changing a scoped pad value moves every pad in the scope by the same delta, with each limit applied separately. A step at a limit does not claim a changed value.

The list targets are `files.scroll`, `projects.scroll`, `settings.row`, `edit.page` and `bank`. The file and project lists clamp at their ends. Settings rows clamp. The MIDI tab uses its visible row list. Pad edit pages wrap as Tab does. FX page selection follows its existing two-slot behavior. Banks clamp as U and O do. A target acts only in its screen or picker. `projects.scroll` acts while the project picker is open, even if the underlying screen ignores ordinary commands. Scrolling Files keeps its current preview behavior. A relative page or bank step creates the same pending-input barrier as a command, so a later pad hit uses the new state.

## Edge cases

| Situation | Expected result | Status | Check |
| --- | --- | --- | --- |
| The owner turns a relative encoder on a newly selected pad | Apply a step to the new pad without pickup. | Support now | Raw MIDI and App state |
| Two devices send the same CC | Keep speed and fine state per port. | Support now | Interleaved timed messages |
| A relative byte decodes to zero | Do not change a value, acknowledge a notice or promote a hold. | Support now | Raw bytes with a held button |
| A fast left turn follows a fast right turn | Use the current direction and a bounded multiplier. | Support now | Timed direction sequence |
| The fine button releases after a screen change or invalid state | Clear fine on its original port and control. | Support now | Raw release sequence |
| The device disconnects during fine mode | Clear fine and speed history. Keep held pad releases. | Support now | Disconnect and reconnect |
| A pad hit follows a page or bank step before UI refresh | The hit uses the new page or bank in order. | Support now | Production dispatch sequence |
| An absolute X-Session knob turns | Keep its old position and takeover behavior. | Preserve | Existing profile tests |
| The SMC-PAD remains in its factory absolute mode | Do not treat its CC values as relative bytes. | New fact needed in 91 | Linux capture |

## Open questions

- Can the owner set the SMC-PAD encoders to relative mode without MidiSuite? Default: do not assume this. Change 91 measures the current bytes and chooses a measured profile or a specified absolute fallback before code.
- Which physical SMC-PAD button can hold `knob.fine` without losing a required direct role? Default: no factory binding in change 90. A slow detent already gives the smallest stored step. Change 91 can add a measured role after change 91 measures its button map.

## Verification

Use raw MIDI bytes through a parsed test profile, seeded roles, dispatch and App state. Supply receipt times in tests. Do not sleep to test acceleration. Test each encoding, both directions, neutral values, two ports, fine release, disconnect, list boundaries, ordered bank changes and absolute-knob parity. Compare the keyboard tap and one slow relative detent on the same parameter. Check the ANSI screen catalogue if a visible row or hint changes. This change needs no audio-thread code or new WAV.
