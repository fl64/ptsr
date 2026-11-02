## Context

See proposal.md for the problem. Change 82 supplies the FX screen and page controls.
This change is planned separately and must not change the owner's existing assignments.

## Goals / Non-Goals

Use the current profile format and existing actions. Test normal MIDI bytes without a
physical controller. Do not program the controller or implement a BLE MIDI transport.
Do not add relative encoders, LED output, aftertouch effects or sequencer functions.

## Sources

- SMC-PAD manufacturer manual: https://manualf.oss-cn-hongkong.aliyuncs.com/manual/MIDI-KEYBOARD/SMC-PAD_ZE.pdf
- SMC-PAD normal MIDI measurements: https://github.com/SDR3078/mvave-home-assistant/blob/main/docs/HARDWARE-BLE.md
- Independent normal preset map: https://github.com/zhrepot/SMC-PAD-for-VirtualDJ/blob/main/README.md
- X-Session manufacturer guide, Appendix A: https://www.manualsdir.com/manuals/122289/m-audio-xsession-pro.html?page=9
- X-Session guide copy: https://manualsnet.com/m-audio/x-session-pro
- Independent X-Session mapping: https://raw.githubusercontent.com/mixxxdj/mixxx/main/res/controllers/M-Audio_Xsession_pro.midi.xml

The manufacturer SMC-PAD manual does not specify every MIDI number. The measured normal
map is a supported setup, not a promise about all presets or USB ports. The owner's unit
must be compared with F12 input before acceptance.

## Decisions

### SMC-PAD supported setup

Use normal Note mode, base notes 36–51 on channel 10, buttons and absolute CC on channel
1. Because the format has one optional channel for the entire device, omit that filter;
test the two channels explicitly. Do not silently introduce per-control channel fields.
Physical PAD1 is bottom-left. Map notes by physical rows: the top row notes 48–51 maps
to screen pads 1–4, then 44–47 to 5–8, 40–43 to 9–12, and 36–39 to 13–16.
Use CC30–37 as knobs 1–8 and CC38–45 as their second hardware set.

| Controls | Pads screen | Chop screen | Edit layer | FX screen |
|---|---|---|---|---|
| Knobs 1–3 | volume, pitch, cutoff | slice start, slice end, cursor | page parameters 1–3 | page parameters 1–3 |
| Knob 4 | no action | zoom | page selector | block selector |
| Knob 5 | no action | region start | no action | no action |
| Knob 6 | no action | region end | no action | no action |
| Knob 7 | no action | hit threshold | no action | no action |
| Knob 8 | master volume | master volume | master volume | master volume |
| Left/Right, CC25/26 | bank previous/next | slice previous/next | page previous/next | block previous/next |
| Play, CC27 | pad edit | play/stop | close edit | no action |
| Stop, CC28 | back | back | close edit | back |
| Record, CC29 | output recording toggle | output recording toggle | output recording toggle | output recording toggle |

The second knob set mirrors these roles with independent control identity. The local
PAD BANK and Shift do not have invented host commands. Notes outside 36–51 are unassigned
in this profile. Do not claim the second pad set changes the ptsr bank.

### X-Session Pro

The X-Session replacement is now part of active change 82. Use its physical control table
and safe defaults. Do not restore the former arbitrary delete or commit buttons. Keep
existing numeric identifiers and preserve initialized user maps until confirmed reset.

Knobs 1/2/3 on the left are CC24/25/26. High/Mid/Low on the left are CC27/28/29.
The corresponding right knobs are CC31–36. Left Pitch is CC12, left Volume CC11,
right Volume CC14, right Pitch CC15. Crossfader CC17 also emits mirrored CC20.
Keep CC20 unassigned to avoid duplicate movement.

Add default fader roles: left Volume controls selected pad volume, left Pitch controls
selected pad pitch, right Volume controls master volume. Right Pitch and the crossfader
remain unassigned because the sampler has no corresponding two-deck mix action.
The page trio controls FX; left backward / forward select the FX block. Change 82 owns
the replacement profile and its single-controller tests. This deferred change adds the
SMC-PAD profile and tests both controllers together.

### Emulation

Use byte sequences through Msg parsing, the shipped TOML, profile seeding, dispatch and
App handling. This emulates the MIDI interface, not the hardware electronics. Cover both
NoteOff forms, velocity, repeated release, button CC127/0 pairs, absolute ramps and ends,
banks, screen switches, two devices, takeover, learning, reset and restart. Pressure and
clock messages must not trigger unrelated actions. No new dependency is needed.

## Migration Plan

New defaults apply only on first setup or confirmed reset. Existing seeded devices and
learned mappings stay unchanged. Test old config round trips. Rollback restores the old
shipped profiles without altering user data.

## Risks / Trade-offs

- Different SMC presets or ports use different maps: document the supported setup and
  verify the actual messages before resetting bindings.
- Endless knobs can still send absolute values: test the stated absolute mode and do
  not enable an unverified relative decoder.
- Visual pad orientation is not note order: verify all sixteen physical positions.
- Hardware feel and actual USB/BLE routing cannot be accepted from byte simulation.

## Open Questions

- Which preset and MIDI port does the owner's SMC-PAD use? Default: the documented normal
  Note/absolute-CC setup above. Hardware acceptance remains open until F12 confirms it.
