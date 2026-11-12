# Design

## Decisions

- **Same piece.** The view builds the piece of an empty pad from `engine::test_tone` of that pad, the same samples the engine plays. The whole tone is the piece, as the source `all` of a sample.
- **No new state.** The view computes the tone and does not save it.

## Open questions

None.
