# Design

## Context and evidence

The connected USB device is `4353:4b4d` (SINCO SMC-PAD). Linux names its ALSA sequencer inputs `SINCO SMC-PAD-Private`, `SINCO SMC-PAD-Master`, and `SINCO`. A simultaneous capture showed the same note and CC messages on Private and Master. The third port sent no message during that capture. The captured pad was channel 10, note 47. The captured encoder was channel 1, CC 33, with absolute values 2 through 8. One captured pad is not proof of every physical position.

Read-only USB SysEx returned active slot index 4, base pad bank index 2, and PAD BANK state zero. A checked read covered the 28,312-byte preset region. Slot 4 has buttons CC 25 through 29 on channel 1. Its encoder records use CC 30 through 45 in absolute mode. Base-bank pads use notes 36 through 51 on channel 10. Second-bank pads use notes 52 through 67 on channel 10. Each reply had valid addresses, lengths and checksums. The local backup has SHA-256 `b3a0f4fe7399a2a4970698be056229bb7f758606d3d8bd1456d704fb3887132d`. No device write was tested.

Sources: [SMC-PAD USB configuration report](https://github.com/spectalive/smc-pad), [SMC-PAD BLE hardware measurements](https://github.com/SDR3078/mvave-home-assistant/blob/main/docs/HARDWARE-BLE.md), [manufacturer manual](https://manualf.oss-cn-hongkong.aliyuncs.com/manual/MIDI-KEYBOARD/SMC-PAD_ZE.pdf), and the earlier source list in `openspec/deferred/83-controller-profiles/design.md`. The source repositories describe a relative encoder record with flag 3, minimum 63 and maximum 65. USB behavior on this unit still needs a write and read-back check.

## Decisions

### Supported preset and two input ports

Support the measured slot and bank map. Use the Master port for performance messages. Load an empty profile for the mirrored Private port, so it cannot make a second pad hit or action. Do not match the third `SINCO` port by its generic name. Keep the device-wide channel filter empty: pads use channel 10 and other controls use channel 1. Keep the owner's saved assignments. A profile is a starting map on first setup or after an explicit reset.

Map physical bottom-left pad 1 to screen bottom-left pad 13. Map notes 48 through 51 to the top screen row. Map notes 44 through 47 to the second row. Map notes 40 through 43 to the third row. Map notes 36 through 39 to the bottom row. Use the same physical row rule for layer 2 notes 52 through 67. Name layer 2 controls `c1` through `c16`. They do not run commands in this change. The first second-layer hit can reveal the panel in change 92. Do not infer a host SHIFT press from the local SHIFT button.

### Encoder setup

The application reads the device state and active encoder records before setup. It may write only the active preset's 16 encoder records, with relative flag 3 and output limits 63 and 65. It must preserve every other byte in the region. It reads the records again and enables relative actions only if all 16 match. It must not use the relative decoder on absolute CC bytes. A failed read, unsupported preset, changed slot, write failure or mismatch leaves encoder actions off. The status names the problem. Pads and buttons keep working. The application saves original records before a write. After a partial write, it tries to restore the saved records and checks them. The status reports a failed restore. Do not change other slots or use a global reset.

The USB write protocol and host access still need a check on this unit. The code step starts with one reversible encoder record and a read-back check. If the record does not match, stop device writes and revise this design before implementation continues. The test suite uses a fake USB transport to check the exact read, write and restore bytes. This test does not replace the hardware read-back check.

### Starting roles and controller view

The five buttons use the planned roles. LEFT and RIGHT select the current list. PLAY starts or continues the current item. STOP stops the transport or preview. RECORD holds microphone recording. No button answers a destructive question. The first knob bank controls the current screen. The second bank controls the selected pad or master value. The controller view names active roles and dims unavailable roles. Existing keyboard actions keep their paths. The command panel and full reach audit are separate changes 92 and 93.

### Test route

Recorded raw MIDI bytes pass through the shipped TOML, production parser, profile selection and App dispatch. Tests cover both note-off forms, velocity, channels, pad layers, relative directions and neutral values. They also cover button pairs, aftertouch, port mirroring, X-Session and unplug during a held gate. The virtual replay command uses these same bytes and profile. Hardware checks use F12 input debug and the actual device. They remain open until a capture records the physical controls.

## Risks and edge cases

| Case | Expected behavior | Status |
| --- | --- | --- |
| Private and Master deliver the same hit | One sound and one action | Mirror observed. Implementation pending. |
| Absolute CC arrives before relative setup | No false large step | Absolute values observed. Implementation pending. |
| Preset changes during setup | Stop write and keep relative actions off | Test pending |
| USB write succeeds in part | Restore saved records and report result | Test pending |
| USB disconnects during a held pad or RECORD | Release the original gate or hold once | Test pending |
| PAD BANK is momentary instead of latched | Use the note number, not a host-side toggle guess | Physical check pending |
| SHIFT sends no host message | Do not assign a SHIFT shortcut | Source report. Physical check pending. |
| Another device uses the same CC numbers | Keep its own profile, state and actions | Test pending |

## Open questions

- Does the owner's USB device accept and retain the relative encoder record? Default: perform one reversible record test, compare read-back, and stop writes if it differs.
- Does PAD BANK latch on this preset? Default: route each received note by its number, so either hardware behavior is safe. The owner check records which behavior occurs.
- Do all physical pads, buttons and both encoder banks emit the values in the preset? Default: ship the read-back map, then compare with a physical F12 capture before hardware acceptance.
