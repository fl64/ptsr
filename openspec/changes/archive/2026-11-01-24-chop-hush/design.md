# Design 24

## Model

- Opening the chop screen sends one "hush the pads" command to the engine; the engine
  fades every pad voice out over the existing cut fade (`FADE_SECONDS`, 5 ms) and frees
  it. The chop playback of the sample is not a pad voice and is not touched.
- One command at the screen switch, not a state: pads hit after the entry sound.

## Decisions (with the owner, 2026-10-30)

- Fade on entry over a "stop all" key: the owner wants silence to listen to the slice
  without an extra press. Cost: a beat cannot keep playing under the chopping.

## Open questions

- The assign mode places pads on a pad hit and plays nothing; unchanged.
