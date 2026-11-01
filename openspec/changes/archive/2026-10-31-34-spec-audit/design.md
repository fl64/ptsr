# Design 34

## Decisions

- The code is right where the main spec quotes an older text and a later requirement of
  the same spec already says the new one ("Chop screen readability": `bank B, pad K: what
  → new`).
- The status prints an ASCII hyphen (`master -3 dB`); the spec follows the program, the
  status line keeps its plain characters.

## Not automated, checked by the owner live

- audio-engine "UI hangs": the audio thread never waits on the UI — architecture, no
  test can hang the UI meaningfully; the real-time rules are checked by the no-allocation
  test of the engine.
- audio-settings "An explicit size still applies", "Server did not accept the buffer",
  "Start without the selected device": they need a running sound server.
- input-devices "Two controllers", "Controller pulled out during play", "Reading the
  layout": they need MIDI ports.

## Open questions

- 21 "Slice editing": the slice under the cursor plays when the cursor stops on it — the
  code selects the slice, it does not play it. Default: nothing changes until the owner
  says whether it should play.
- 21 "Chopping from the X-Session Pro": the scenario promises the cursor and the region
  from the controller; the profile maps only the previous and next slice. Default:
  nothing changes until the owner says which buttons.
